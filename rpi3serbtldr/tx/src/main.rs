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
    let matches = App::new("rpi3serbtldr_px")
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
             .required(false))
        .arg(Arg::with_name("timeout")
             .help("Time out in milliseconds eg. 2000 (two seconds)")
             .short("t")
             .use_delimiter(false)
             .takes_value(true)
             .required(false))
        .arg(Arg::with_name("jtag")
             .help("Signals target to initialize JTAG (if applicable)")
             .short("j")
             .use_delimiter(false)
             .takes_value(false)
             .required(false))
        .arg(Arg::with_name("wait")
             .help("Signals target to go into an infinite loop.")
             .short("w")
             .use_delimiter(false)
             .takes_value(false)
             .required(false))
        .arg(Arg::with_name("echo")
             .help("After jump to loaded code echo output.")
             .short("e")
             .use_delimiter(false)
             .takes_value(false)
             .required(false))
        .get_matches();

//Get values passed from command line or defaults
    let port_name = matches.value_of("port").unwrap();
    let baud_rate = matches.value_of("baud").unwrap_or("115200");
    let fname     = matches.value_of("file").unwrap_or("");
    let time_out  = matches.value_of("timeout").unwrap_or("2000");
    let jtag      = matches.is_present("jtag");
    let wait      = matches.is_present("wait");
    let echo      = matches.is_present("echo");

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

//Tell user what will happen.
    println!("");
    println!("rpi3serbtldr_tx");
    println!("---------------");
    println!("File: {}", &fname);
    println!("Port: \"{}\"", &port_name);
    println!("Baud: {}", &baud_rate);
    println!("Timeout(ms): {0}", &time_out);
    println!("JTAG: {}", if jtag { "Yes" } else { "No" } );
    println!("Wait: {}", if wait { "Yes" } else { "No" } );
    println!("Read and Echo: {}", if echo { "Yes" } else { "No" } );
    println!("");
    println!("Begin...");

//Open serial port.
    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
//Wait for remote port to transmit break signal.                    
            match wait_for_break_signal(&mut port) {
                Ok(()) => {
                    if jtag {
                        if let Err(_) = send_jtag(&mut port) {
                            eprintln!("Failed to send jtag instruction.");
                        }
                    }

                    if fname.len() > 0 {
                        match File::open(fname) {
                            Ok(mut file) => {
                                if let Err(_) = send_load(&mut file, &mut port) {
                                    eprintln!("Failed to send data.");
                                } else {
                                    if let Err(_) = send_jump(&mut port) {
                                        eprintln!("Failed to send jump.");
                                    }
                                    
                                    if echo {
                                        read_and_echo(&mut port);
                                    } else {
                                        return;
                                    }
                                }
                            },

                            Err(e) => {
                                eprintln!("Failed to open file \"{}\". Error: {}", fname, e);
                            }
                        }
                    }

                    if wait {
                        if let Err(_) = send_wait(&mut port) {
                            eprintln!("Failed to send wait instruction.");
                        } else {
                            if echo {
                                read_and_echo(&mut port);
                            } else {
                                return;
                            }
                        }
                    }
                },

                Err(e) => {
                    eprintln!("Break signal not received. Error: {}", e);
                }
            }
        },

        Err(e) => {
            eprintln!("Failed to open port \"{}\". Error: {}", port_name, e);
        }
    }
}

///
/// Send the contents of an open file over the port.
///
/// #Arguments
///
/// * `file` - an initialized and opened file.
/// * `port` - an initialized and opened serial port to write to.
///
fn send_load(file: &mut File, port: &mut PortType) -> Result <(),()> {
    let instr: [u8;4] = serialize_u32((0x4C4F4144 as u32).to_le()); //'L','O','A','D'

    if let Err(e) = write_bytes(port, &instr) {
        eprintln!("Failed to send LOAD instruction. Error: {}", e);
        return Err(());
    }

    if let Err(e) = send_file_size(file, port) {
        eprintln!("Failed to send file size. Error: {}", e);
        return Err(());
    }

    match wait_for_ok_signal(port) {
        Ok(ok) => {
            if ok == false {
                return Err(());
            }
        },

        Err(e) => {
            eprintln!("OK signal not received after sending file size. Error: {}", e);
            return Err(());
        }
    }

    if let Err(e) = send_file(file, port) {
        eprintln!("Failed to send file. Error: {}", e);
        return Err(());
    }


    match wait_for_ok_signal(port) {
        Ok(ok) => {
            if ok == false {
                return Err(());
            }
        },

        Err(e) => {
            eprintln!("OK signal not received after sending file. Error: {}", e);
            return Err(());
        }
    }

    Ok(())
}

fn send_jump(port: &mut PortType) -> Result <(),()> {
    let instr: [u8;4] = serialize_u32((0x4A554D50 as u32).to_le()); //'J','U','M','P'

    eprintln!("Send JUMP instruction.");
    
    if let Err(e) = write_bytes(port, &instr) {
        eprintln!("Failed to send JUMP instruction. Error: {}", e);
        return Err(());
    }

    Ok(())
}


///
/// Send a JTAG signal over the port.
///
/// #Arguments
///
/// * `port` - an initialized and opened serial port to write to.
///
fn send_jtag(port: &mut PortType) -> Result <(),()> {
    let instr: [u8;4] = serialize_u32((0x4A544147 as u32).to_le()); //'J','T','A','G'

    eprintln!("Send JTAG instruction.");

    if let Err(e) = write_bytes(port, &instr) {
        eprintln!("Failed to send JTAG instruction. Error: {}", e);
        return Err(());
    }

    match wait_for_ok_signal(port) {
        Ok(ok) => {
            if ok == false {
                return Err(());
            }
        },

        Err(e) => {
            eprintln!("OK signal not received after sending JTAG instruction. Error: {}", e);
            return Err(());
        }
    }

    Ok(())
}


///
/// Send a WAIT signal over the port.
///
/// #Arguments
///
/// * `port` - an initialized and opened serial port to write to.
///
fn send_wait(port: &mut PortType) -> Result <(),()> {
    let instr: [u8;4] = serialize_u32((0x57414954 as u32).to_le()); //'W','A','I','T'

    eprintln!("Send WAIT instruction.");

    if let Err(e) = write_bytes(port, &instr) {
        eprintln!("Failed to send WAIT instruction. Error: {}", e);
        return Err(());
    }

    match wait_for_ok_signal(port) {
        Ok(ok) => {
            if ok == false {
                return Err(());
            }
        },

        Err(e) => {
            eprintln!("OK signal not received after sending WAIT instruction. Error: {}", e);
            return Err(());
        }
    }

    Ok(())
}


///
/// Send the contents of an open file over the port. Return a CRC.
///
/// #Arguments
///
/// * `file` - an initialized and opened file.
/// * `port` - an initialized and opened serial port to write to.
///
fn send_file(file: &mut File, port: &mut PortType) -> Result <(), Error> {
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

    println!("Received break signal.");
    return Ok(());
}

///
/// Waits for an OK signal sent by the client on the hardware.
///
/// Returns ErrorKind::TimedOut if port read exceeded timeout and/or number of tries.
///         ErrorKind::* if there was another error.
///         Ok(true) - Got OK.
///         Ok(false) - Got ER.
///
/// #Arguments
///
/// * `port` - an initialized and opened serial port to read from.
///
fn wait_for_ok_signal(port: &mut PortType) -> Result<bool, Error>  {
    let cmplst: [u8; 2] = ['O' as u8, 'E' as u8];
    let mut ok = false;

    match read_cmp_byte(port, &cmplst) {
        Ok(ch) => {
            if ch == 'O' as u8 { //Might be 'OK'. 
                let cmpch: [u8; 1] = ['K' as u8];
                match read_cmp_byte(port, &cmpch) {
                    Ok(_) => {
                        println!("Got OK signal.");
                        ok = true;
                    },
                    
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            if ch == 'E' as u8 { //Might be 'ER'.
                let cmpch: [u8; 1] = ['R' as u8];
                match read_cmp_byte(port, &cmpch) {
                    Ok(_) => {
                        println!("Got ER (error) signal.");
                        ok = false;
                    },

                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        },

        Err(e) => {
            return Err(e);
        }
    }
    
    Ok(ok)
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
                },

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
            },

            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                print!("Timed out while trying to write port.");
            },

            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                print!("Interrupted while trying to write port.");
            },

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
///         Matching byte in list.
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
            },
 
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                print!("Timed out while trying to read port.");
            },

            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                print!("Interrupted while trying to read port.");
            },
            
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


///
/// Reads a byte from the serial port and echoes it to the console.
///
/// #Arguments
///
/// * `port` - an initialized and opened serial port to read from.
///
fn read_and_echo(port: &mut PortType) {
    let mut buf: [u8; 1] = [0x00];

    println!("rpi3serbtldr_tx - Read port and echo output.");

    loop {
        match port.read(&mut buf) {
            Ok(amt_rd) if amt_rd == 1 => {
                print!("{}", buf[0] as char);
            },

            Ok(_) => {},

            Err(ref e) if e.kind() == ErrorKind::TimedOut => {},

            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                println!("rpi3serbtldr_tx - Interrupted while trying to read port. Quitting.");
                break;
            },
            
            Err(_) => {
                break;
            }
        }
    }
}
