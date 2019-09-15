extern crate clap;
extern crate serialport;

use std::io::ErrorKind;
use std::io::Error;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use clap::{App, AppSettings, Arg};
use serialport::prelude::*;

type PortType = Box<dyn serialport::SerialPort>; //Easier than typing the whole thing out.

const NUM_RETRIES: u8 = 10; //Number of times to retry a port read/write operation.


fn main() {
    let matches = App::new("rpi3serbtldr_tx")
        .about("Transmit a file via serial port to a rpi3serbtldr client.")
        .setting(AppSettings::DisableVersion)
        .arg(Arg::with_name("port")
             .help("The device path to a serial port eg. /dev/ttyACM0")
             .short("p")
             .use_delimiter(false)
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("baud")
             .help("Baudrate of the serial connection eg. 115200")
             .short("b")
             .use_delimiter(false)
             .takes_value(true)
             .required(false))
        .arg(Arg::with_name("file")
             .help("Name of file to transmit eg. ./kernel8.img")
             .short("f")
             .use_delimiter(false)
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("timeout")
             .help("Time out in milliseconds eg. 2000 (two seconds)")
             .short("t")
             .use_delimiter(false)
             .takes_value(true)
             .required(false))
        .get_matches();

//Get values passed from command line or defaults
    let port_name = matches.value_of("port").unwrap();
    let baud_rate = matches.value_of("baud").unwrap_or("115200");
    let file_name = matches.value_of("file").unwrap();
    let time_out  = matches.value_of("timeout").unwrap_or("2000");
    let mut settings: SerialPortSettings = Default::default();


//Determine if time_out is an actual number and assign to the serial port settings.      
    if let Ok(timeout) = time_out.parse::<u64>() {
        settings.timeout = Duration::from_millis(timeout);
    } else {
        eprintln!("Error: Invalid timeout '{}' specified", &time_out);
        ::std::process::exit(1);
    }

//Determine if baud_rate is an actual number and assign to serial port settings.  
    if let Ok(rate) = baud_rate.parse::<u32>() {
        settings.baud_rate = rate.into();
    } else {
        eprintln!("Error: Invalid baud rate '{}' specified", &baud_rate);
        ::std::process::exit(1);
    }

    println!("");
    println!("rpi3serbtldr_tx");
    println!("---------------");
    println!("File: {}", &file_name);
    println!("Port: \"{}\"", &port_name);
    println!("Baud: {}", &baud_rate);
    println!("Timeout(ms): {0}", &time_out);
    println!("");
    println!("Begin...");

//Open file to send over serial port.
    match File::open(file_name) {
        Ok(mut file) => {
//Open serial port.
            match serialport::open_with_settings(&port_name, &settings) {
                Ok(mut port) => {
//Wait for remote port to transmit break signal.                    
                    match wait_for_break_signal(&mut port) {
                        Ok(()) => {
//Send the size of the file to the remote port.                            
                            match send_file_size(&mut file, &mut port) {
                                Ok(_) => {
//Wait for remote port to transmit OK signal.                                    
                                    match wait_for_ok_signal(&mut port) {
                                        Ok(_) => {
//Send the contents of the file to the remote port.
                                            match send_file(&mut file, &mut port) {
                                                Ok(_crc) => {
//Wait for the remote port to transmit OK signal.
                                                    match wait_for_ok_signal(&mut port) {
                                                        Ok(_) => {
                                                            println!("File sent successfully. Read and echo replies.");
                                                            read_and_echo(&mut port);
                                                        }

                                                        Err(e) => {
                                                            eprintln!("Send file OK signal not received. Error: {}", e);
                                                        }
                                                    }
                                                }

                                                Err(e) => {
                                                    eprintln!("Failed to send file. Error: {}", e);
                                                }
                                            }
                                        }

                                        Err(e) => {
                                            eprintln!("OK signal not received. Error: {}", e);
                                        }
                                    }
                                }

                                Err(e) => {
                                    eprintln!("Failed to send file size. Error: {}", e);
                                }
                            }
                        },

                        Err(e) => {
                            eprintln!("Break signal not received. Error: {}", e);
                        }
                    }
                }

                Err(e) => {
                    eprintln!("Failed to open port \"{}\". Error: {}", port_name, e);
                }
            }
        }

        Err(e) => {
            eprintln!("Failed to open file \"{}\". Error: {}", file_name, e);
        }
    }
}


///
/// Send the contents of an open file over the port. Return a CRC.
///
/// #Arguments
///
/// * `file` - an initialized and opened file.
/// * `port` - an initialized and opened serial port to write to.
///
fn send_file(file: &mut File, port: &mut PortType) -> Result < u32, Error> {
    let mut crc: u32 = 0;
    let mut buf: [u8; 1] = [0x00];
    let mut fbuf = Vec::new();

    match file.read_to_end(&mut fbuf) {
        Ok(_) => {
            let mut i = 0;
            println!("Sending file.");
            for b in fbuf.iter() {
                buf[0] = *b;
                match write_bytes(port, &buf) {
                    Ok(_) => {
                        i = i + 1;
                        //println!("Sent: {} of {}", i, fbuf.len())
                    }

                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }

        Err(e) => {
            return Err(e);
        }
    }

    return Ok(crc);  //FIXME: For now return zero for the CRC.
}


///
/// Waits for an OK signal sent by the client on the hardware.
///
/// Returns ErrorKind::TimedOut if port read exceeded timeout and/or number of tries.
///         ErrorKind::* if there was another error.
///         OKSignal::OK - Ready to send file.
///         OKSignal::SizeExceeded - File too big to fit in remote device's memory.
///
/// #Arguments
///
/// * `port` - an initialized and opened serial port to read from.
///
fn wait_for_ok_signal(port: &mut PortType) -> Result<(), Error>  {
    let cmplst: [u8; 2] = ['O' as u8, 'S' as u8];

    match read_cmp_byte(port, &cmplst) {
        Ok(ch) => {        
            if ch == 'O' as u8 { 
                let cmpch: [u8; 1] = ['K' as u8];            
                match read_cmp_byte(port, &cmpch) {
                    Ok(_) => {
                        println!("Got OK signal.");
                        return Ok(());
                    },
                    
                    Err(e) => {
                        return Err(e);
                    }
                }
            } else {
                let cmpch: [u8; 1] = ['E' as u8];            
                match read_cmp_byte(port, &cmpch) {
                    Ok(_) => {
                        return Err(
                            Error::new(ErrorKind::Other, "File size exceeds amount of memory available in device.")
                        );
                    },
                    
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
        
        Err(e) => {
            return Err(e)
        }
    }
}


///
/// Serialize a u32 into a 4 byte array.
///
/// Returns 4 byte array containing u32.
///
/// #Arguments
///
/// * `x` - u32 to serialize.
///
fn serialize_u32(x:u32) -> [u8;4] {
    return[(x & 0xff)         as u8,
           ((x >> 8)  & 0xff) as u8,
           ((x >> 16) & 0xff) as u8,
           ((x >> 24) & 0xff) as u8];
}


/*
///
/// Deserialize a 4 byte array into a u32.
///
/// Returns deserialized u32.
///
/// #Arguments
///
/// * `bytes` - 4 byte array containing serialized u32.
///
fn deserialize_u32(bytes: [u8;4]) -> u32 {
    let mut i: u32 = bytes[0] as u32;
    i |= (bytes[1] as u32) << 8;
    i |= (bytes[2] as u32) << 16;
    i |= (bytes[3] as u32) << 24; 
    return i;
}
*/


///
/// Send the size of an open file over the port as a u32 in little endian format.
///
/// #Arguments
///
/// * `port` - an initialized and opened serial port to write to.
/// * `file` - an initialized and opened file.
///
fn send_file_size(file: &mut File, port: &mut PortType) -> Result< (), Error > {
    match file.metadata() {
        Ok(md) => {
            let len: u32 = md.len() as u32;
            let buf: [u8; 4] = serialize_u32(len.to_le());
            
            match write_bytes(port, &buf) {
                Ok(_) => {
                    println!("Sent file size: {}.", len);
                }
                
                Err(e) => {
                    return Err(e);
                }
            }
        },

        Err(e) => {
            return Err(e);
        }
    }
    
    return Ok(());
}


///
/// Waits for a break signal sent by the client on the hardware.
///
/// Returns ErrorKind::TimedOut if port read exceeded timeout and/or number of tries.
///         ErrorKind::* if there was another error.
///         Ok() if break signal read.
///
/// #Arguments
///
/// * `port` - an initialized and opened serial port to read from.
///
fn wait_for_break_signal(port: &mut PortType) -> Result< (), Error > {
    let cmplst: [u8; 1] = [0x03];
    let mut i = 0;
    
    while i < 3 {
        match read_cmp_byte(port, &cmplst) {
            Ok(ch) if ch == 0x03 => { i = i + 1; },
            Err(ref e) if e.kind() == ErrorKind::NotFound => { i = 0; }
            Err(e) => return Err(e),
            _ => ()
        }
    }

    println!("Receieved break signal.");
    return Ok(());
}


///
/// Writes an array of bytes to the port.
///
/// Returns ErrorKind::TimedOut if write operation exceeded port's timeout or number of retries.
///         ErrorKind::* if there was another error.
///         Ok() on successful write.
///
/// #Arguments
///
/// * `port` - an initialized and opened serial port to read from.
/// * `bytes` - reference to an initialed array of bytes to write.
///
fn write_bytes(port: &mut PortType, bytes: &[u8] ) -> Result<(), Error> {

    for retries in 0..NUM_RETRIES {
        match port.write(bytes) {
            Ok(amt_wr) if amt_wr == bytes.len() => {
                return Ok(());
            }

            Ok(_) => {
                return Err(
                    Error::new(ErrorKind::Other, "Got unexpected amount while trying to write port.")
                );
            }

            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                print!("Timed out while trying to write port.");
            }

            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                print!("Interrupted while trying to write port.");
            }

            Err(e) => {
                return Err(e);
            }
        }

        println!(" {} retries left.", NUM_RETRIES - retries);
    }

    return Err( 
        Error::new(ErrorKind::TimedOut, "Exceeded number of retries while trying to write port.")
    );
}


///
/// FIXME: Might be better to split this into two functions. read_byte() and byte_cmp().
///
/// Reads a byte from the serial port and compares it to a list of bytes.
///
/// Returns ErrorKind::NotFound if received byte doesn't match list.
///         ErrorKind::TimedOut if read operation exceeded port's timeout or number of retries.
///         ErrorKind::* if there was another error.
///         Matching character in list.
///
/// #Arguments
///
/// * `port` - an initialized and opened serial port to read from.
/// * `cmplst` - reference to an initialed array of bytes to compare against.
///
fn read_cmp_byte(port: &mut PortType, cmplst: &[u8] ) -> Result< u8, Error > {
    let mut buf: [u8; 1] = [0x00];

    for retries in 0..NUM_RETRIES {
        match port.read(&mut buf) {
            Ok(amt_rd) if amt_rd == 1 => { //Read one byte. Match to cmplst.
                for ch in cmplst {
                    if *ch == buf[0] {
                        return Ok(*ch);
                    }
                }
                return Err (
                    Error::new (
                        ErrorKind::NotFound, 
                        format! (
                            "Got unexpected byte {}: '{}' while trying to read port.", 
                            buf[0], buf[0] as char
                        )
                    )
                );
            },
            
            Ok(amt_rd) => { //Anything else is considered an error.
                return Err (
                    Error::new (
                        ErrorKind::Other, 
                        format! (
                            "Got unexpected amount {} while trying to read port.", 
                            amt_rd
                        )
                    )
                );
            }
 
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                print!("Timed out while trying to read port.");
            }

            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                print!("Interrupted while trying to read port.");
            }
            
            Err(e) => {
                return Err(e);
            }
        }

        println!(" {} retries left.", NUM_RETRIES - retries);
    }

    return Err( 
        Error::new(ErrorKind::TimedOut, "Exceeded number of retries while trying to read port.")
    );
}

fn read_and_echo(port: &mut PortType) {
    let mut buf: [u8; 1] = [0x00];

    loop {
        match port.read(&mut buf) {
            Ok(amt_rd) if amt_rd == 1 => {
                print!("{}", buf[0] as char);
            }

            Ok(_) => {}

            Err(ref e) if e.kind() == ErrorKind::TimedOut => {}

            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                println!("rpi3serbtldr_tx - Interrupted while trying to read port. Quitting.");
                break;
            }
            
            Err(_) => {
                break;
            }
        }
    }
}
