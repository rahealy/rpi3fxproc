/*
 * MIT License
 *
 * Copyright (c) 2019 Richard Healy
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

/*
 * Slave address: 
 * 100111x0 Read
 * 100111x1 Write
 *
 */

use register::register_bitfields;
use register::mmio::ReadWrite;
use peripherals::{debug, i2c};

/**********************************************************************
 * ERROR
 *********************************************************************/

pub enum ERROR {
    I2C(i2c::ERROR),
    WRONGID
}

impl ERROR {
    pub fn msg (&self) -> &'static str {
        match self {
            ERROR::I2C(err) => err.msg(),
            ERROR::WRONGID => "Device returned the wrong chip ID."
        }
    }
}


/**********************************************************************
 * CS4265 Registers
 *********************************************************************/

register_bitfields! {
    u8,

///I2C address of the CS4265 depends on whether the CS4265 SDOUT pin is
///pulled high or low.
    CHIPADDR [
        ADDR OFFSET(1) NUMBITS(7)
    ],

///Chip ID register. I2C address 0x01.
    CHIPID [
        PART OFFSET(4) NUMBITS(4) [
///CS4265 I2C Chip ID
            ID = 0b1101
        ],
///Revision level
        REV  OFFSET(0) NUMBITS(4) [
            A    = 0b0001,
            B_C0 = 0b0010,
            C1   = 0b0011
        ]
    ],
///Power control register. I2C address 0x02.
    POWERCTL [
///Freeze control, set POWERCTL flags, unfreeze to apply parameters.
        FREEZE  OFFSET(7) NUMBITS(1) [],
///Power down microphone.
        PDN_MIC OFFSET(3) NUMBITS(1) [],
///Power down ADC
        PDN_ADC OFFSET(2) NUMBITS(1) [],
///Power down DAC
        PDN_DAC OFFSET(1) NUMBITS(1) [],
///Power down
        PDN     OFFSET(0) NUMBITS(1) []
    ],
///DAC Control register 1. I2C address 0x03.
    DACCTL1 [
///Digital interface format.
        DAC_DIF OFFSET(4) NUMBITS(2) [
///Left justified up to 24bit data
            LJ24BIT  = 0b00,
///I2S up to 24 bit data.
            I2S24BIT = 0b01,
///Right justified fixed 16bit length.
            RJ16BIT  = 0b10,
///Right justified fixed 24 bit length.
            RJ24BIT  = 0b11
        ],
///Mute the DAC.
        MUTEDAC OFFSET(2) NUMBITS(1) [],
///De-emphasis filter.
        DEEMPH OFFSET(1) NUMBITS(1) []
    ],
///ADC Control register. I2C address 0x04.
    ADCCTL [
///Functional Mode.
        FM OFFSET (6) NUMBITS(2) [
            SINGLE_SPEED_4_50_KHZ   = 0b00,
            DOUBLE_SPEED_50_100_KHZ = 0b01,
            QUAD_SPEED_100_200_KHZ  = 0b10
        ],
///ADC Digital Interface Format
        ADC_DIF   OFFSET (4) NUMBITS(1) [
            LJ24BIT  = 0b0,
            I2S24BIT = 0b1
        ],
///ADC Mute
        MUTEADC OFFSET (2) NUMBITS(1) [],
///High pass filter freeze.
        HPFFREEZE OFFSET (1) NUMBITS(1) [],
///Master / Slave mode for serial audio port (LJ24BIT or I2S24BIT)
        MS OFFSET (0) NUMBITS(1) []
    ],
///MCLK register. I2C address 0x05.
    MCLK [
///MCLK frequency divider
        MCLK OFFSET(4) NUMBITS(3) [
///Divide by 1.0
            DIV1_0 = 0b000,
///Divide by 1.5
            DIV1_5 = 0b001,
///Divide by 2.0
            DIV2_0 = 0b010,
///Divide by 3.0
            DIV3_0 = 0b011,
///Divide by 4.0
            DIV4_0 = 0b100
        ]
    ],
///Signal selection register. I2C address 0x06.
    SIGSEL [
///Set the serial audio data source for the DAC.
        SDINSEL OFFSET(7) NUMBITS(1) [
            SDIN1 = 0b0,
            SDIN2 = 0b1
        ],
///Digital loopback.
        LOOP OFFSET(1) NUMBITS(1) []
    ],
///Programmable gain amplifier Channel B register. I2C address 0x07.
    PGAB [
///Gain
        GAIN OFFSET(0) NUMBITS(6) []
    ],
///Programmable gain amplifier Channel A register. I2C address 0x08.
    PGAA [
///Gain
        GAIN OFFSET(0) NUMBITS(6) []
    ],
///Analog input control register. I2C address 0x09.
    AICTL [
///Soft ramp muting.
        PGASOFT OFFSET(4) NUMBITS(1) [],
///Zero cross muting.
        PGAZERO OFFSET(3) NUMBITS(1) [],
///Input source select
        SELECT  OFFSET(0) NUMBITS(1) [
///Microphone level
            MIC  = 0b0,
///Line level
            LINE = 0b1
        ]
    ],
///DAC volume channel A register. I2C address 0x0A.
    DACVOLA [
        VOL OFFSET(0) NUMBITS(8) []
    ],
///DAC volume channel B register. I2C address 0x0B.
    DACVOLB [
        VOL OFFSET(0) NUMBITS(8) []
    ],
///DAC Control register 2. I2C address 0x0C.
    DACCTL2 [
///Soft ramp muting
        DACSOFT   OFFSET(7) NUMBITS(1) [],
///Zero cross muting
        DACZERO   OFFSET(6) NUMBITS(1) [],
///Invert DAC output.
        INVERTDAC OFFSET(5) NUMBITS(1) []
    ],
///Status register. I2C address 0x0D.
    STATUS [
///Completion of an E to F C-Buffer translation. Refer to datasheet.
        EFTC        OFFSET(4) NUMBITS(1) [],
///Clock error.
        CLKERR      OFFSET(3) NUMBITS(1) [],
///ADC overflow condition.
        ADCOVFL     OFFSET(1) NUMBITS(1) [],
///ADC underflow condition.
        ADCUNDRFL   OFFSET(0) NUMBITS(1) []
    ],
///Status mask register. I2C address 0x0E.
    STATUSMASK [
///Select which conditions affect the STATUS register. 
        EFTCM       OFFSET(4) NUMBITS(1) [],
        CLKERRM     OFFSET(3) NUMBITS(1) [],
        ADCOVFLM    OFFSET(1) NUMBITS(1) [],
        ADCUNDRFLM  OFFSET(0) NUMBITS(1) []
    ],
///Status mode most significant bit register. I2C address 0x0F.
/// The combination of STATUSMODEMSB and STATUSMODELSB determines when
/// the status bit is set in Status:
/// 0b00 = status bit set on rising edge of condition.
/// 0b01 = status bit set on falling edge of condition clear.
/// 0b10 = status bit set as long as condition is active.
    STATUSMODEMSB [
        EFTC1       OFFSET(4) NUMBITS(1) [],
        CLKERR1     OFFSET(3) NUMBITS(1) [],
        ADCOVFL1    OFFSET(1) NUMBITS(1) [],
        ADCUNDRFL1  OFFSET(0) NUMBITS(1) []
    ],
///Status mode least significant bit register. I2C address 0x10.
    STATUSMODELSB [
        EFTC0       OFFSET(4) NUMBITS(1) [],
        CLKERR0     OFFSET(3) NUMBITS(1) [],
        ADCOVFL0    OFFSET(1) NUMBITS(1) [],
        ADCUNDRFL0  OFFSET(0) NUMBITS(1) []
    ],
///Transmitter control register 1. I2C address 0x11.
    XMITCTL1 [
///E to F C-data buffer transfers are inhibited when set.
        EFTCI   OFFSET(6) NUMBITS(1) [],
///When set operates in 2 byte mode. When cleared 1 byte mode.
        CAM     OFFSET(5) NUMBITS(1) []
    ],
///Transmitter control register 2. I2C address 0x12.
    XMITCTL2 [
///Transmitter digital interface format.
        TX_DIF  OFFSET(6) NUMBITS(2) [
///Left justified up to 24bit data
            LJ24BIT  = 0b00,
///I2S up to 24 bit data.
            I2S24BIT = 0b01,
///Right justified fixed 16bit length.
            RJ16BIT  = 0b10,
///Right justified fixed 24 bit length.
            RJ24BIT  = 0b11
        ],
///Turns off transmitter when set.
        TXOFF   OFFSET(5) NUMBITS(1) [],
///When set transmitter will output zeros.
        TXMUTE  OFFSET(4) NUMBITS(1) [],
///Indicates valid PCM data when set.
        V       OFFSET(3) NUMBITS(1) [],
///When set transmitter outputs mono.
        MMT     OFFSET(2) NUMBITS(1) [],
///When clear channel A and B are output in separate frames. When set
///MMTLR value determines which channel is output in both frames.
        MMTCS   OFFSET(1) NUMBITS(1) [],
///If MMTCS is set and MMTLR is clear then channel A else channel B.
        MMTLR   OFFSET(0) NUMBITS(1) []
    ]
}


/**********************************************************************
 * CS4265 I2C Bus Addresses
 *********************************************************************/

#[allow(non_snake_case)]
#[repr(u8)]
///7 bit i2c address
pub enum Address {
///Address if CS4265 SDOUT pin is pulled low to gnd.
    LOW  = 0b1001110,
///Address if CS4265 SDOUT pin has a pullup resistor.
    HIGH = 0b1001111
}


/**********************************************************************
 * CS4265 I2C Register Addresses
 *********************************************************************/

#[allow(non_snake_case)]
#[repr(u8)]
pub enum RegisterAddress {
///Chip ID register. I2C address 0x01.
    CHIPID = 0x01,
///Power control register. I2C address 0x02.
    POWERCTL = 0x02,
///DAC Control register 1. I2C address 0x03.
    DACCTL1 = 0x03,
///ADC Control register. I2C address 0x04.
    ADCCTL = 0x04,
///MCLK register. I2C address 0x05.
    MCLK = 0x05,
///Signal selection register. I2C address 0x06.
    SIGSEL = 0x06,
///Programmable gain amplifier Channel B register. I2C address 0x07.
    PGAB = 0x07,
///Programmable gain amplifier Channel A register. I2C address 0x08.
    PGAA = 0x08,
///Analog input control register. I2C address 0x09.
    AICTL = 0x09,
///DAC volume channel A register. I2C address 0x0A.
    DACVOLA = 0x0A,
///DAC volume channel B register. I2C address 0x0B.
    DACVOLB = 0x0B,
///DAC Control register 2. I2C address 0x0C.
    DACCTL2 = 0x0C,
///Status register. I2C address 0x0D.
    STATUS = 0x0D,
///Status mask register. I2C address 0x0E.
    STATUSMASK = 0x0E,
///Status mode most significant bit register. I2C address 0x0F.
    STATUSMODEMSB = 0x0F,
///Status mode least significant bit register. I2C address 0x10.
    STATUSMODELSB = 0x10,
///Transmitter control register 1. I2C address 0x11.
    XMITCTL1 = 0x11,
///Transmitter control register 2. I2C address 0x12.
    XMITCTL2 = 0x12
}


/**********************************************************************
 * CS4265 In-Memory Representation Access
 *********************************************************************/

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
///I2C address of the CS4265 0x00.
    CHIPADDR: ReadWrite<u8, CHIPADDR::Register>,

///Chip ID register. I2C address 0x01.
    CHIPID: ReadWrite<u8, CHIPID::Register>,

///Power control register. I2C address 0x02.
    POWERCTL: ReadWrite<u8, POWERCTL::Register>,

///DAC Control register 1. I2C address 0x03.
    DACCTL1: ReadWrite<u8, DACCTL1::Register>,

///ADC Control register. I2C address 0x04.
    ADCCTL: ReadWrite<u8, ADCCTL::Register>,

///MCLK register. I2C address 0x05.
    MCLK: ReadWrite<u8, MCLK::Register>,

///Signal selection register. I2C address 0x06.
    SIGSEL: ReadWrite<u8, SIGSEL::Register>,

///Programmable gain amplifier Channel B register. I2C address 0x07.
    PGAB: ReadWrite<u8, PGAB::Register>,

///Programmable gain amplifier Channel A register. I2C address 0x08.
    PGAA: ReadWrite<u8, PGAA::Register>,

///Analog input control register. I2C address 0x09.
    AICTL: ReadWrite<u8, AICTL::Register>,

///DAC volume channel A register. I2C address 0x0A.
    DACVOLA: ReadWrite<u8, DACVOLA::Register>,

///DAC volume channel B register. I2C address 0x0B.
    DACVOLB: ReadWrite<u8, DACVOLB::Register>,

///DAC Control register 2. I2C address 0x0C.
    DACCTL2: ReadWrite<u8, DACCTL2::Register>,

///Status register. I2C address 0x0D.
    STATUS: ReadWrite<u8, STATUS::Register>,

///Status mask register. I2C address 0x0E.
    STATUSMASK: ReadWrite<u8, STATUSMASK::Register>,

///Status mode most significant bit register. I2C address 0x0F.
    STATUSMODEMSB: ReadWrite<u8, STATUSMODEMSB::Register>,

///Status mode least significant bit register. I2C address 0x10.
    STATUSMODELSB: ReadWrite<u8, STATUSMODELSB::Register>,

///Transmitter control register 1. I2C address 0x11.
    XMITCTL1: ReadWrite<u8, XMITCTL1::Register>,

///Transmitter control register 2. I2C address 0x12.
    XMITCTL2: ReadWrite<u8, XMITCTL2::Register>
}


/**********************************************************************
 * CS4265 In-Memory Register Representation
 *********************************************************************/

///
/// CS4265 in memory local representations of peripheral registers 
/// accessed via I2C
///
#[allow(non_snake_case)]
#[derive(Default)]
pub struct CS4265<S> {
    i2c: S,

///I2C address of the CS4265 0x00.
    pub CHIPADDR: u8,

///Chip ID register. I2C address 0x01.
    pub CHIPID: u8,

// ///Power control register. I2C address 0x02.
//     POWERCTL: u8,
// 
// ///DAC Control register 1. I2C address 0x03.
//     DACCTL1: u8,
// 
// ///ADC Control register. I2C address 0x04.
//     ADCCTL: u8,
// 
// ///MCLK register. I2C address 0x05.
//     MCLK:  u8,
// 
// ///Signal selection register. I2C address 0x06.
//     SIGSEL:  u8,
// 
// ///Programmable gain amplifier Channel B register. I2C address 0x07.
//     PGAB: u8,
// 
// ///Programmable gain amplifier Channel A register. I2C address 0x08.
//     PGAA:  u8,
// 
// ///Analog input control register. I2C address 0x09.
//     AICTL: u8,
// 
// ///DAC volume channel A register. I2C address 0x0A.
//     DACVOLA:  u8,
// 
// ///DAC volume channel B register. I2C address 0x0B.
//     DACVOLB: u8,
// 
// ///DAC Control register 2. I2C address 0x0C.
//     DACCTL2: u8,
// 
// ///Status register. I2C address 0x0D.
//     STATUS: u8,
// 
// ///Status mask register. I2C address 0x0E.
//     STATUSMASK: u8,
// 
// ///Status mode most significant bit register. I2C address 0x0F.
//     STATUSMODEMSB: u8,
// 
// ///Status mode least significant bit register. I2C address 0x10.
//     STATUSMODELSB: u8,
// 
// ///Transmitter control register 1. I2C address 0x11.
//     XMITCTL1: u8,
// 
// ///Transmitter control register 2. I2C address 0x12.
//     XMITCTL2: u8
}

impl <I> CS4265<I> where
    I: i2c::I2C + Default
{
    pub fn new() -> CS4265<I> {
        CS4265 { ..Default::default() }
    }

///
///Poll chip at given address for the chip id.
///
    pub fn poll_chip_id(&mut self, addr: u8) -> Result<(), ERROR> {
        let mut chipid: [u8;1] = [0];

        match self.i2c.read(addr,
                            RegisterAddress::CHIPID as u8,
                            &mut chipid)
        {
            Ok(())   => {
                self.CHIPID = chipid[0];
    
                if ((self.CHIPID & 0b11110000) >> 4) == 0b1101 {
                    debug::out("cs4265.poll_chip_id(): Found Part ID 0b1101. Ok.\r\n");

                    match self.CHIPID & 0b00001111 {
                        0b0001 => {
                            debug::out("cs4265.poll_chip_id(): Found Part Revision A.\r\n");
                        },
                        0b0010 => {
                            debug::out("cs4265.poll_chip_id(): Found Part Revision B or C0.\r\n");
                        },
                        0b0011 => {
                            debug::out("cs4265.poll_chip_id(): Found Part Revision C1.\r\n");
                        },
                        _ => {
                            debug::out("cs4265.poll_chip_id(): Found Unknown Part Revision.\r\n");
                        }
                    }
                    Ok(())
                } else {
                    Err(ERROR::WRONGID)
                }

            },

            Err(err) => {
                Err(ERROR::I2C(err))
            }
        }
    }

///
/// Assumes cs4265 input pin RESET has been driven high and enough time
/// has passed for MCLK to settle.
///
/// Poll for cs4265 address and chipid.
///
    pub fn poll(&mut self) -> Result<(), ERROR> {
        debug::out("cs4265.poll(): Trying LOW address...\r\n");
        match self.poll_chip_id(Address::LOW as u8) {
            Ok(_) => {
                debug::out("cs4265.poll(): cs4265 found at LOW address.\r\n");
                self.CHIPADDR = Address::LOW as u8;
                return Ok(());
            }

            Err(_) => {
                debug::out("cs4265.poll(): Poll for LOW address failed.\r\n");
            }
        }

        debug::out("cs4265.poll(): Trying HIGH address...\r\n");
        match self.poll_chip_id(Address::HIGH as u8) {
            Ok(_) => {
                debug::out("cs4265.poll(): cs4265 found at HIGH address.\r\n");
                self.CHIPADDR = Address::HIGH as u8;
                return Ok(());
            }

            Err(err) => {
                debug::out("cs4265.poll(): Poll for HIGH address failed.\r\n");
                return Err(err);
            }
        }
    }
    
    pub fn init(&mut self) -> Result<(), ERROR> {
        debug::out("cs4265.init(): Initializing CS4265.\r\n");
//Poll address and chip id.
        if let Err(err) = self.poll() {
            return Err(err);
        }

//Set power control register. Turn on DAC and ADC.
        
        match self.i2c.read(self.CHIPADDR,
                            RegisterAddress::POWERCTL as u8,
                            &mut chipid)
        {
        }
        debug::out("cs4265.init(): CS4265 initialized.\r\n");
        return Ok(());
    }
}
