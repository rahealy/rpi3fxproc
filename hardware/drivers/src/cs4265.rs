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
use core::mem;
use core::ops;

/**********************************************************************
 * ERROR
 *********************************************************************/

pub enum ERROR {
    I2C(i2c::ERROR),
    WRONGID,
    STATUS(&'static str),
    RANGE,
}

impl ERROR {
    pub fn msg (&self) -> &'static str {
        match self {
            ERROR::I2C(err) => err.msg(),
            ERROR::WRONGID => "Device returned the wrong chip ID.",
            ERROR::STATUS(err) => err,
            ERROR::RANGE => "Argument is out of accepted range."
        }
    }
}


/**********************************************************************
 * CS4265 Registers
 *********************************************************************/

register_bitfields! {
    u8,

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
        DIV OFFSET(4) NUMBITS(3) [
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
        GAIN OFFSET(0) NUMBITS(6) []
    ],
///Programmable gain amplifier Channel A register. I2C address 0x08.
    PGAA [
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
//Volume reduction in 1/2 dB increments (0..255)
        VOL OFFSET(0) NUMBITS(8) [],
///Set this bit for 1/2 dB.
        HALF OFFSET(0) NUMBITS(1) [],
///Two's complement integer value 0dB..-127dB.
        INT OFFSET(1) NUMBITS(7) []
    ],
///DAC volume channel B register. I2C address 0x0B.
    DACVOLB [
//Volume reduction in 1/2 dB increments (0..255)
        VOL OFFSET(0) NUMBITS(8) [],
///Set this bit for 1/2 dB.
        HALF OFFSET(0) NUMBITS(1) [],
///Integer value 0dB..-127dB.
        INT OFFSET(1) NUMBITS(7) []
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
#[derive(Copy,Clone)]
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
///Chip ID register. I2C address 0x01.
    pub CHIPID: ReadWrite<u8, CHIPID::Register>,

///Power control register. I2C address 0x02.
    pub POWERCTL: ReadWrite<u8, POWERCTL::Register>,

///DAC Control register 1. I2C address 0x03.
    pub DACCTL1: ReadWrite<u8, DACCTL1::Register>,

///ADC Control register. I2C address 0x04.
    pub ADCCTL: ReadWrite<u8, ADCCTL::Register>,

///MCLK register. I2C address 0x05.
    pub MCLK: ReadWrite<u8, MCLK::Register>,

///Signal selection register. I2C address 0x06.
    pub SIGSEL: ReadWrite<u8, SIGSEL::Register>,

///Programmable gain amplifier Channel B register. I2C address 0x07.
    pub PGAB: ReadWrite<u8, PGAB::Register>,

///Programmable gain amplifier Channel A register. I2C address 0x08.
    pub PGAA: ReadWrite<u8, PGAA::Register>,

///Analog input control register. I2C address 0x09.
    pub AICTL: ReadWrite<u8, AICTL::Register>,

///DAC volume channel A register. I2C address 0x0A.
    pub DACVOLA: ReadWrite<u8, DACVOLA::Register>,

///DAC volume channel B register. I2C address 0x0B.
    pub DACVOLB: ReadWrite<u8, DACVOLB::Register>,

///DAC Control register 2. I2C address 0x0C.
    pub DACCTL2: ReadWrite<u8, DACCTL2::Register>,

///Status register. I2C address 0x0D.
    pub STATUS: ReadWrite<u8, STATUS::Register>,

///Status mask register. I2C address 0x0E.
    pub STATUSMASK: ReadWrite<u8, STATUSMASK::Register>,

///Status mode most significant bit register. I2C address 0x0F.
    pub STATUSMODEMSB: ReadWrite<u8, STATUSMODEMSB::Register>,

///Status mode least significant bit register. I2C address 0x10.
    pub STATUSMODELSB: ReadWrite<u8, STATUSMODELSB::Register>,

///Transmitter control register 1. I2C address 0x11.
    pub XMITCTL1: ReadWrite<u8, XMITCTL1::Register>,

///Transmitter control register 2. I2C address 0x12.
    pub XMITCTL2: ReadWrite<u8, XMITCTL2::Register>
}

#[repr(C)]
#[derive(Default)]
pub struct RegisterInstance {
    data: [u8; mem::size_of::<RegisterBlock>()]
}

impl ops::Deref for RegisterInstance {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe {
            mem::transmute::<&RegisterInstance, &RegisterBlock>(self)
        }
    }
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
    addr: u8,
    pub reg: RegisterInstance
}

impl <I> CS4265<I> where
    I: i2c::I2C + Default
{
    
///
///Initialize the CS4265.
///
    pub fn init(&mut self) -> Result<(), ERROR> {
        debug::out("cs4265.init(): Initializing CS4265.\r\n");
        if let Err(err) = self.poll() { //Poll address and chip id.
            return Err(err);
        }
        
        if let Err(err) = self.rd_regs() { //Read register defaults.
            return Err(err);
        }

        debug::out("cs4265.init(): CS4265 initialized.\r\n");
        return Ok(());
    }
///
///Modify the clock and divider for sample rates.
///FIXME: Only supports MCLK value of 12.288 MHz
///
/// Clock table for CS4265
/// 
/// Given Ultra2 MCLK of 12288000 Hz:
///
/// Single Speed (ADCCTL::FM::SINGLE_SPEED_4_50_KHZ)
/// Fs      MCLK  Divider
/// ------------------------------------------------
/// 32kHz,  384x, 1.5 (MCLK::DIV::DIV1_5)
/// 48kHz,  256x, 1.0 (MCLK::DIV::DIV1_0)
/// 
/// Double Speed (ADCCTL::FM::DOUBLE_SPEED_50_100_KHZ)
/// --------------------------------------------------
/// 64kHz,  192x, 1.5 (MCLK::DIV::DIV1_5)
/// 96kHz,  128x, 1.0 (MCLK::DIV::DIV1_0)
/// 
/// Quad Speed (ADCCTL::FM::QUAD_SPEED_100_200_KHZ)
/// -----------------------------------------------
/// 128kHz, 96x, 1.5 (MCLK::DIV::DIV1_5)
/// 192kHz, 64x, 1.0 (MCLK::DIV::DIV1_0)
///
    pub fn modify_clk(&self, fs_hz: u32, mclk_hz: u32) -> Result<(), ERROR> {
        if mclk_hz == 12_288_000 {
            if fs_hz == 0 { //Set to lowest possible speed.
                debug::out("cs4265.modify_clk(): SINGLE_SPEED_4_50_KHZ & DIV4_0.\r\n");
                self.reg.ADCCTL.modify(ADCCTL::FM::SINGLE_SPEED_4_50_KHZ);
                self.reg.MCLK.modify(MCLK::DIV::DIV4_0);
            } else if fs_hz == 32_000 {
                debug::out("cs4265.modify_clk(): SINGLE_SPEED_4_50_KHZ & DIV1_5.\r\n");
                self.reg.ADCCTL.modify(ADCCTL::FM::SINGLE_SPEED_4_50_KHZ);
                self.reg.MCLK.modify(MCLK::DIV::DIV1_5);
            } else if fs_hz == 48_000 {
                debug::out("cs4265.modify_clk(): SINGLE_SPEED_4_50_KHZ & DIV1_0.\r\n");
                self.reg.ADCCTL.modify(ADCCTL::FM::SINGLE_SPEED_4_50_KHZ);
                self.reg.MCLK.modify(MCLK::DIV::DIV1_0);
            } else if fs_hz == 64_000 {
                debug::out("cs4265.modify_clk(): DOUBLE_SPEED_50_100_KHZ & DIV1_5.\r\n");
                self.reg.ADCCTL.modify(ADCCTL::FM::DOUBLE_SPEED_50_100_KHZ);
                self.reg.MCLK.modify(MCLK::DIV::DIV1_5);
            } else if fs_hz == 96_000 {
                debug::out("cs4265.modify_clk(): DOUBLE_SPEED_50_100_KHZ & DIV1_0.\r\n");
                self.reg.ADCCTL.modify(ADCCTL::FM::DOUBLE_SPEED_50_100_KHZ);
                self.reg.MCLK.modify(MCLK::DIV::DIV1_0);
            } else if fs_hz == 128_000 {
                debug::out("cs4265.modify_clk(): QUAD_SPEED_100_200_KHZ & DIV1_5.\r\n");
                self.reg.ADCCTL.modify(ADCCTL::FM::QUAD_SPEED_100_200_KHZ);
                self.reg.MCLK.modify(MCLK::DIV::DIV1_5);
            } else if fs_hz == 192_000 {
                debug::out("cs4265.modify_clk(): QUAD_SPEED_100_200_KHZ & DIV1_0.\r\n");
                self.reg.ADCCTL.modify(ADCCTL::FM::QUAD_SPEED_100_200_KHZ);
                self.reg.MCLK.modify(MCLK::DIV::DIV1_0);
            } else {
                return Err(ERROR::RANGE);
            }
        } else {
            return Err(ERROR::RANGE);
        }

        return Ok(());
    }

///
///Load the register value into the CS4265.
///
    pub fn ld_reg(&self, ra: RegisterAddress) -> Result<(), ERROR> {

        match self.i2c.write(self.addr, 
                             ra as u8,
                             &self.reg.data[(ra as usize) - 1..ra as usize]) 
        {
            Ok(_) => {
                return Ok(());
            },

            Err(err) => {
                return Err(ERROR::I2C(err));
            }
        }
    }

///
///Read the register value from the CS4265.
///
    pub fn rd_reg(&mut self, ra: RegisterAddress) -> Result<(), ERROR> {
        match self.i2c.read(self.addr, 
                            ra as u8,
                            &mut self.reg.data[(ra as usize) - 1..ra as usize]) 
        {
            Ok(_) => {
                return Ok(());
            },

            Err(err) => {
                return Err(ERROR::I2C(err));
            }
        }
    }

///
///load all the local settings into the CS4265's registers.
///
    pub fn ld_regs(&self) -> Result<(), ERROR> {
//Load settings.
        match self.i2c.write(self.addr,
                             RegisterAddress::POWERCTL as u8, 
                             &self.reg.data[(RegisterAddress::POWERCTL as usize) - 1..])
        {
            Ok(_) => {
                return Ok(());
            },

            Err(err) => {
                return Err(ERROR::I2C(err));
            }
        }
    }

///
///Read all the CS4265's registers into local memory.
///
    pub fn rd_regs(&mut self) -> Result<(), ERROR> {
        match self.i2c.read(self.addr,
                            RegisterAddress::POWERCTL as u8, 
                            &mut self.reg.data[(RegisterAddress::POWERCTL as usize) - 1..])
        {
            Ok(_) => {
                return Ok(());
            },

            Err(err) => {
                return Err(ERROR::I2C(err));
            }
        }
    }

///
///Print the contents of the registers similarlly to the linux i2cdump command.
///
    pub fn dump_regs(&self) {
        let mut cur: RegisterInstance = RegisterInstance::default();

        if let Err(err) = self.i2c.read(self.addr,
                                        RegisterAddress::CHIPID as u8, 
                                        &mut cur.data) 
        {
            debug::out("cs4265.dump_regs(): Error: ");
            debug::out(err.msg());
        }
        
        debug::out("     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f\r\n");
        debug::out("00:    ");
        for i in 0x01..0x10 {
            debug::u8hexi(cur.data[i - 1]);
            debug::out(" ");
        }
        debug::out("\r\n10: ");
        for i in 0x10..0x13 {
            debug::u8hexi(cur.data[i - 1]);
            debug::out(" ");
        }
        debug::out("\r\n");
    }

///
///Read all the registers from the device and verify they match the state.
///
    pub fn verify_regs(&self) -> Result<(), ERROR> {
        let mut cur: RegisterInstance = RegisterInstance::default();

        if let Err(err) = self.i2c.read(self.addr,
                                        RegisterAddress::POWERCTL as u8, 
                                        &mut cur.data[(RegisterAddress::POWERCTL as usize) - 1..])
        {
            return Err(ERROR::I2C(err));
        }
        
        for i in (RegisterAddress::POWERCTL as usize) - 1..cur.data.len() {
            debug::out("cs4265.verify(): ");
            debug::u8hex((i + 1) as u8);
            if self.reg.data[i] != cur.data[i] {
                debug::out(" Bad  - ");
            } else {
                debug::out(" Good - ");
            }
            debug::out("Expected / Got: "); 
            debug::u8bits(self.reg.data[i]);
            debug::out("/");
            debug::u8bits(cur.data[i]);
            debug::out("\r\n");
        }

        return Ok(());
    }

    pub fn print_status(&mut self) -> Result<(), ERROR> {
         
        if let Err(err) = self.rd_reg(RegisterAddress::STATUS) {
            return Err(err);
        }

        if self.reg.STATUS.is_set(STATUS::EFTC) {
            debug::out("cs4265.print_status(): Completion of an E to F C-Buffer translation.\r\n");
        }

        if self.reg.STATUS.is_set(STATUS::CLKERR) {
            debug::out("cs4265.print_status(): Clock error.\r\n");
        }
    
        if self.reg.STATUS.is_set(STATUS::ADCOVFL) {
            debug::out("cs4265.print_status(): ADC overflow condition.\r\n");
        }
        
        if self.reg.STATUS.is_set(STATUS::ADCUNDRFL) {
            debug::out("cs4265.print_status(): ADC underflow condition.\r\n");
        }
        
        debug::out("cs4265.print_status(): Done.\r\n");
        return Ok(());
    }

///
///Poll chip at given address for the chip id.
///
    fn poll_chip_id(&mut self, addr: u8) -> Result<(), ERROR> {
        let mut chipid: [u8;1] = [0];

        match self.i2c.read(addr,
                            RegisterAddress::CHIPID as u8,
                            &mut chipid)
        {
            Ok(())   => {
                self.reg.CHIPID.set(chipid[0]);
    
                if self.reg.CHIPID.matches_all(CHIPID::PART::ID) {
                    debug::out("cs4265.poll_chip_id(): Found Part ID 0b1101. Ok.\r\n");

                    match chipid[0] & 0b00001111 {
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
    fn poll(&mut self) -> Result<(), ERROR> {
        debug::out("cs4265.poll(): Trying LOW address...\r\n");
        match self.poll_chip_id(Address::LOW as u8) {
            Ok(_) => {
                debug::out("cs4265.poll(): cs4265 found at LOW address.\r\n");
                self.addr = Address::LOW as u8;
                return Ok(());
            },

            Err(_) => {
                debug::out("cs4265.poll(): Poll for LOW address failed.\r\n");
            }
        }

        debug::out("cs4265.poll(): Trying HIGH address...\r\n");
        match self.poll_chip_id(Address::HIGH as u8) {
            Ok(_) => {
                debug::out("cs4265.poll(): cs4265 found at HIGH address.\r\n");
                self.addr = Address::HIGH as u8;
                return Ok(());
            },

            Err(err) => {
                debug::out("cs4265.poll(): Poll for HIGH address failed.\r\n");
                return Err(err);
            }
        }
    }
}
