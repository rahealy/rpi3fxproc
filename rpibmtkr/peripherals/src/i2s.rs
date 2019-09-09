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

use super::MMIO_BASE;
use core::ops;
use register::register_bitfields;
use register::mmio::ReadWrite;


/**********************************************************************
 * ERROR
 *********************************************************************/

///
///Module error definitions.
///
pub enum ERROR {
///
    TIMEOUT
}

impl ERROR {
    pub fn msg (&self) -> &'static str {
        match self {
            ERROR::TIMEOUT   => "Unexpected timeout."
        }
    }
}

/**********************************************************************
 * GPIO
 *
 * Reference:
 *  https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
 *
 * I2S Pins
 *  RPi I/O Pin 12, BCM GPIO18, Function: PCM_CLK, I2S: BCLK
 *  RPi I/O Pin 35, BCM GPIO19, Function: PCM_FS, I2S: LRCLK
 *  RPi I/O Pin 38, BCM GPIO20, Function: PCM_DIN, I2S: SDIN
 *  RPi I/O Pin 40, BCM GPIO21, Function: PCM_DOUT, I2S: SDOUT
 *
 *********************************************************************/

register_bitfields! {
    u32,

/// GPIO Function Select 1
    GPFSEL1 [
/// I/O Pin 12 (BCLK)
        FSEL18 OFFSET(24) NUMBITS(3) [
            PCM_CLK = 0b100 // I2S - Alternate function 0
        ],

/// I/O Pin 35 (LRCLK)
        FSEL19 OFFSET(27) NUMBITS(3) [
            PCM_FS = 0b100 // I2S - Alternate function 0
        ]
    ],

/// GPIO Function Select 2
    GPFSEL2 [
/// I/O Pin 38 (SDIN)
        FSEL20 OFFSET(0) NUMBITS(3) [
            PCM_DIN = 0b100 // I2S - Alternate function 0
        ],

/// I/O Pin 40 (SDOUT)
        FSEL21 OFFSET(3) NUMBITS(3) [
            PCM_DOUT = 0b100 // I2S - Alternate function 0
        ]
    ]
}

///
///GPFSEL1 alternative function select register - 0x7E200004
///
const GPFSEL1_OFFSET: u32 = 0x0020_0004;
const GPFSEL1_BASE:   u32 = MMIO_BASE + GPFSEL1_OFFSET;

// ///
// ///GPFSEL2 alternative function select register - 0x7E200008
// ///
// const GPFSEL2_OFFSET: u32 = 0x0020_0008;
// const GPFSEL2_BASE:   u32 = MMIO_BASE + GPFSEL2_OFFSET;

///
/// GPFSEL peripheral registers
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockGPFSEL {
    GPFSEL1: ReadWrite<u32, GPFSEL1::Register>, // 0x00200004
    GPFSEL2: ReadWrite<u32, GPFSEL2::Register>  // 0x00200008
}

///
/// GPFSEL peripheral registers
///
pub struct GPFSEL;

impl ops::Deref for GPFSEL {
    type Target = RegisterBlockGPFSEL;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl GPFSEL {
    fn ptr() -> *const RegisterBlockGPFSEL {
        GPFSEL1_BASE as *const _
    }

    pub const fn new() -> GPFSEL {
        GPFSEL
    }

    pub fn fsel_i2s(&self) {
        self.GPFSEL1.modify(GPFSEL1::FSEL18::PCM_CLK + 
                            GPFSEL1::FSEL19::PCM_FS);

        self.GPFSEL2.modify(GPFSEL2::FSEL20::PCM_DIN + 
                            GPFSEL2::FSEL21::PCM_DOUT);
    }
}

/**********************************************************************
 * PCMDIV
 *
 * Reference
 *  https://www.scribd.com/doc/127599939/BCM2835-Audio-clocks
 *********************************************************************/

register_bitfields! {
    u32,

///PCM clock divider. 0x7E10_109C
    CM_PCMDIV [
///Enter password before changing a value.
        PASSWD OFFSET(24) NUMBITS(8) [
            VAL = 0x5A
        ],

///Integer part of divisor.
        DIVI OFFSET(12) NUMBITS(12) [],

///Fractional part of divisor.
        DIVF OFFSET(0) NUMBITS(12) []
    ]
}

///
///PCM clock divider. 0x7E10_109C
///
const CM_PCMDIV_OFFSET: u32 = 0x0010_109C;
const CM_PCMDIV_BASE:   u32 = MMIO_BASE + CM_PCMDIV_OFFSET; 

///Clock divder.
struct PCMDIV;

impl ops::Deref for PCMDIV {
    type Target = ReadWrite<u32, CM_PCMDIV::Register>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl PCMDIV {
    pub const fn new() -> PCMDIV {
        PCMDIV
    }

    fn ptr() -> *const ReadWrite<u32, CM_PCMDIV::Register> {
        CM_PCMDIV_BASE as *const _
    }

///
///Reference
/// https://github.com/arisena-com/rpi_src/blob/master/apps/i2s_test/src/i2s_test.c
///
/// Set 12 bit integer and fractional values of divider.
///
    fn set(&self, i: u32, f: u32) {
        self.modify (
            CM_PCMDIV::PASSWD::VAL  +
            CM_PCMDIV::DIVI.val(i) + //Integer divisor of 10
            CM_PCMDIV::DIVF.val(f)   //Fractional divisor of 1/4095
        );
    }
}

/**********************************************************************
 * PCMCTL
 *
 * Reference
 *  https://www.scribd.com/doc/127599939/BCM2835-Audio-clocks
 *********************************************************************/

register_bitfields! {
    u32,

///PCM clock control. 0x7E10_1098
    CM_PCMCTL [
///Enter password before changing a value.
        PASSWD OFFSET(24) NUMBITS(8) [
            VAL = 0x5A
        ],

///MASH control
        MASH OFFSET(9) NUMBITS(2) [
            INT   = 0b00,   //Integer division
            ONE   = 0b01,   //One stage MASH
            TWO   = 0b10,   //Two stage MASH
            THREE = 0b11    //Three stage MASH
        ],

///Debug only. Generates edge on clock output.
        FLIP OFFSET(8) NUMBITS(1) [],

///Clock generator running.
        BUSY OFFSET(7) NUMBITS(1) [],

///Kill and restart clock generator. Debug only.
        KILL OFFSET(5) NUMBITS(1) [],

///Enable clock generator. Poll BUSY bit for result.
        ENAB OFFSET(4) NUMBITS(1) [],

///Clock source.
        SRC OFFSET(0) NUMBITS(4) [
            GND        = 0b0000,    //Ground (no clock)
            OSC        = 0b0001,    //19.2 MHz oscillator
            TESTDEBUG0 = 0b0010,    //???
            TESTDEBUG1 = 0b0011,    //???
            PLLA       = 0b0100,    //Phase locked loop A
            PLLC       = 0b0101,    //Phase locked loop C
            PLLD       = 0b0110,    //Phase locked loop D
            HDMIAUX    = 0b0111     //HDMI Auxillary
        ]
    ]
}


///
///PCM (I2C) clock control. 0x7E10_1098
///
const CM_PCMCTL_OFFSET: u32 = 0x0010_1098;
const CM_PCMCTL_BASE:   u32 = MMIO_BASE + CM_PCMCTL_OFFSET; 


///Clock control.
struct PCMCTL;

impl ops::Deref for PCMCTL {
    type Target = ReadWrite<u32, CM_PCMCTL::Register>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl PCMCTL {
    pub const fn new() -> PCMCTL {
        PCMCTL
    }

    fn ptr() -> *const ReadWrite<u32, CM_PCMCTL::Register> {
        CM_PCMCTL_BASE as *const _
    }

    fn wait_busy(&self, val: bool) {
        while self.is_set(CM_PCMCTL::BUSY) != val {}
    }

///
///Reference
/// https://github.com/arisena-com/rpi_src/blob/master/apps/i2s_test/src/i2s_test.c
///
    pub fn init(&self) {
//Clear clock register. Wait until stop.
        self.write(CM_PCMCTL::PASSWD::VAL);

        self.wait_busy(false);

//Set divider based on control values.
        self.modify (
            CM_PCMCTL::PASSWD::VAL + //Password.
            CM_PCMCTL::MASH::INT   + //MASH set to integer.
            CM_PCMCTL::SRC::OSC      //Use oscillator for clock source.
        );

//Set divider frequency to 19.2 MHz / (10 + (1/4095)) = 1.9199531147 MHz
        PCMDIV::new().set(10, 1);

//Keep the control values used to set divider and enable. Wait until started.
        self.modify (
            CM_PCMCTL::PASSWD::VAL + //Password.
            CM_PCMCTL::MASH::INT   + //MASH set to integer.
            CM_PCMCTL::SRC::OSC    + //Use oscillator for clock source.
            CM_PCMCTL::ENAB::SET
        );

        self.wait_busy(true);
    }
}

/**********************************************************************
 * PCM
 *
 * Reference:
 *  https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
 *********************************************************************/
register_bitfields! {
    u32,

///CS_A control and status. Offset 0x0.
    CS_A [
///Standby. Set to exit standby. Takes at least 4 PCM clocks.
        STBY OFFSET(25) NUMBITS(1) [],

///Permit software to address sync issues. Takes 2 PCM clocks.
        SYNC OFFSET(24) NUMBITS(1) [],

///Sign extend sample data (repeat most significant bit to fill 32bit value).
        RXSEX OFFSET(23) NUMBITS(1) [],

///FIFO is full and will overflow.
        RXF OFFSET(22) NUMBITS(1) [],

///FIFO is empty and will underflow.
        TXE OFFSET(21) NUMBITS(1) [],

///FIFO contains data.
        RXD OFFSET(20) NUMBITS(1) [],

///FIFO is not full and can be written to.
        TXD OFFSET(19) NUMBITS(1) [],

///FIFO is above threshold and needs reading.
        RXR OFFSET(18) NUMBITS(1) [],

///FIFO is below threshold and needs writing.
        TXW OFFSET(17) NUMBITS(1) [],

///Under or overflow error.
        RXERR OFFSET(16) NUMBITS(1) [],

///Under or overflow error.
        TXERR OFFSET(15) NUMBITS(1) [],

///FIFO is in sync with data frame.
        RXSYNC OFFSET(14) NUMBITS(1) [],

///FIFO is in sync with data frame.
        TXSYNC OFFSET(13) NUMBITS(1) [],

///Enable DMA access.
        DMAEN OFFSET(9) NUMBITS(1) [],

///Threshold determines when the RXR flag is set.
        RXTHR OFFSET(7) NUMBITS(2) [
            A  = 0b00, //FIXME: BCM Documentation unclear.
            B  = 0b01, //FIXME: BCM Documentation unclear.
            C  = 0b10, //FIXME: BCM Documentation unclear.
            D  = 0b11  //FIXME: BCM Documentation unclear.
        ],

///Threshold determines when the TXW flag is set.
        TXTHR OFFSET(5) NUMBITS(2) [
            A  = 0b00, //FIXME: BCM Documentation unclear.
            B  = 0b01, //FIXME: BCM Documentation unclear.
            C  = 0b10, //FIXME: BCM Documentation unclear.
            D  = 0b11  //FIXME: BCM Documentation unclear.            
        ],

///Clear the RX FIFO. Takes 2 PCM Clocks.
        RXCLR OFFSET(4) NUMBITS(1) [],

///Clear the TX FIFO. Takes 2 PCM Clocks.
        TXCLR OFFSET(3) NUMBITS(1) [],

///Enable transmit.
        TXON OFFSET(2) NUMBITS(1) [],

///Enable receive.
        RXON OFFSET(1) NUMBITS(1) [],

///Enable PCM interface.
        EN OFFSET(1) NUMBITS(1) []
    ],

///FIFO Buffer. Offset 0x4.
    FIFO_A [
        DATA OFFSET(0) NUMBITS(32) []
    ],

///Mode. Offset 0x8.
    MODE_A [
///Disable PCM Clock.
        CLK_DIS OFFSET(28) NUMBITS(1) [],

///PDM decimation factor.
        PDMN OFFSET(27) NUMBITS(1) [
            DF16 = 0b0,
            DF32 = 0b1
        ],

///PDM input mode enable.
        PDME OFFSET(26) NUMBITS(1) [
            PCM = 0b0,
            PDM = 0b1
        ],

///Pack 2x16bit samples into one 32bit FIFO location.
        FRXP OFFSET(25) NUMBITS(1) [],

///Pack 2x16bit samples into one 32bit FIFO location. 
        FTXP OFFSET(24) NUMBITS(1) [],

///Clock mode.
        CLKM OFFSET(23) NUMBITS(1) [
            MASTER = 0b0,
            SLAVE  = 0b1
        ],

///Clock invert.
        CLKI OFFSET(22) NUMBITS(1) [
            IFALLING_ORISING = 0b0, //Inputs sampled on falling edge, output on rising edge. 
            IRISING_OFALLING = 0b1  //Inputs sampled on rising edge, output on falling edge. 
        ],

///Frame sync mode.
        FSM OFFSET(21) NUMBITS(1) [
            MASTER = 0b0,
            SLAVE  = 0b1            
        ],

///Frame sync invert.
        FSI OFFSET(20) NUMBITS(1) [
            LOW_TO_HIGH = 0b0, //Sync frame on low to high transition.
            HIGH_TO_LOW = 0b1  //Sync frame on high to low transition.
        ],

///Frame length in clocks - 1 when FSM is master. 1 = 2 clocks, 2 = 3 clocks...
        FLEN OFFSET(10) NUMBITS(10) [],

///Frame sync length in clocks when FSM is master. 0 = off, 1 = one clock, ...
        FSLEN OFFSET(0) NUMBITS(10) []
    ],

///Receive configuration. Offset 0xC.
    RXC_A [
///Width extension bit allows >24 bit samples.
        CH1WEX OFFSET(31) NUMBITS(1) [],

///Channel enable.
        CH1EN OFFSET(30) NUMBITS(1) [],

///Channel position in clocks in frame.
        CH1POS OFFSET(20) NUMBITS(10) [],

///Sample size in bits 0x0 = 8 bits, 0xF = 24 bits.
        CH1WID OFFSET(16) NUMBITS(4) [
            W8  = 0x0,
            W16 = 0x8,
            W24 = 0xF
        ],

///Width extension bit allows >24 bit samples.
        CH2WEX OFFSET(15) NUMBITS(1) [],

///Channel enable.
        CH2EN OFFSET(14) NUMBITS(1) [],

///Channel position in clocks in frame.
        CH2POS OFFSET(4) NUMBITS(10) [],

///Sample size in bits 0x0 = 8 bits, 0xF = 24 bits.
        CH2WID OFFSET(0) NUMBITS(4) [
            W8  = 0x0,
            W16 = 0x8,
            W24 = 0xF
        ]
    ],

///Transmit configuration. Offset 0x10.
    TXC_A [
///Width extension bit allows >24 bit samples.
        CH1WEX OFFSET(31) NUMBITS(1) [],

///Channel enable.
        CH1EN OFFSET(30) NUMBITS(1) [],

///Channel position in clocks in frame. 0 is first clock.
        CH1POS OFFSET(20) NUMBITS(10) [],

///Sample size in bits 0x0 = 8 bits, 0xF = 24 bits.
        CH1WID OFFSET(16) NUMBITS(4) [
            W8  = 0x0,
            W16 = 0x8,
            W24 = 0xF
        ],

///Width extension bit allows >24 bit samples.
        CH2WEX OFFSET(15) NUMBITS(1) [],

///Channel enable.
        CH2EN OFFSET(14) NUMBITS(1) [],

///Channel position in clocks in frame. 0 is first clock.
        CH2POS OFFSET(4) NUMBITS(10) [],

///Sample size in bits 0x0 = 8 bits, 0xF = 24 bits.
        CH2WID OFFSET(0) NUMBITS(4) [
            W8  = 0x0,
            W16 = 0x8,
            W24 = 0xF
        ]
    ],

///DMA Request level. Offset 0x14.
    DREQ_A [
///FIFO DMA Panic level.
        TX_PANIC OFFSET(24) NUMBITS(7) [],

///FIFO DMA Panic level.
        RX_PANIC OFFSET(16) NUMBITS(7) [],

///Request level. When below this level PCM will request more data.
        TX OFFSET(8) NUMBITS(7) [],

///Request level. When below this level PCM will request more data.
        RX OFFSET(0) NUMBITS(7) []
    ],

///Interupt enable. Offset 0x18.
    INTEN_A [
///Enable interupts on FIFO error.
        RXERR OFFSET(3) NUMBITS(1) [],

///Enable interupts on FIFO error.
        TXERR OFFSET(2) NUMBITS(1) [],

///Enable interrupts when FIFO level is >= threshold.
        RXR OFFSET(1) NUMBITS(1) [],

///Enable interrupts when FIFO level is <= threshold.
        TXW OFFSET(0) NUMBITS(1) []
    ],

///Interrupt status and clear. Offset 0x1C.
    INTSC_A [
///FIFO error.
        RXERR OFFSET(3) NUMBITS(1) [],

///FIFO error.
        TXERR OFFSET(2) NUMBITS(1) [],

///FIFO level is >= threshold.
        RXR OFFSET(1) NUMBITS(1) [],

///FIFO level is <= threshold.
        TXW OFFSET(0) NUMBITS(1) []
    ],

///Gray mode control. Offset 0x20.
    GRAY [
///Number of words currently in FIFO
        RXFIFOLEVEL OFFSET(16) NUMBITS(6) [],       

///Number of bits that were flushed on flush operation.
        FLUSHED OFFSET(10) NUMBITS(6) [],       

///Number of GRAY coded bits have been received.
        RXLEVEL OFFSET(4) NUMBITS(6) [],       

///Flush the GRAY RX buffer into the PCM RX FIFO.
        FLUSH OFFSET(2) NUMBITS(1) [],

///Clear GRAY mode logic and flush the RX buffer.
        CLR OFFSET(1) NUMBITS(1) [],

///Enable GRAY mode.
        EN OFFSET(0) NUMBITS(1) []
    ]
}


///
///PCM registers control the I2S peripheral. 0x7E203000.
///
const PCM_OFFSET:  u32 = 0x0020_3000;
const PCM_BASE:    u32 = MMIO_BASE + PCM_OFFSET; 


///
/// PCM peripheral registers
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockPCM {
///CS_A control and status. Offset 0x0.
    CS_A:       ReadWrite<u32, CS_A::Register>,

///FIFO Buffer. Offset 0x4.
    FIFO_A:     ReadWrite<u32, FIFO_A::Register>,

///Mode. Offset 0x8.
    MODE_A:     ReadWrite<u32, MODE_A::Register>,

///Receive configuration. Offset 0xC.
    RXC_A:      ReadWrite<u32, RXC_A::Register>,

///Transmit configuration. Offset 0x10.
    TXC_A:      ReadWrite<u32, TXC_A::Register>,

///DMA Request level. Offset 0x14.
    DREQ_A:     ReadWrite<u32, DREQ_A::Register>,

///Interupt enable. Offset 0x18.
    INTEN_A:    ReadWrite<u32, INTEN_A::Register>,

///Interrupt status and clear. Offset 0x1C.
    INTSC_A:    ReadWrite<u32, INTSC_A::Register>,

///Gray mode control. Offset 0x20.
    GRAY:       ReadWrite<u32, GRAY::Register>
}


///
/// PCM Parameters
///
#[derive(Default)]
pub struct Channel {
    en:  u32, //Channel enable.
    wid: u32, //Bit depth.
    wex: u32,
    pos: u32, //Position in frame.
}

impl Channel {
    pub fn enable(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.en = if val { 1 } else { 0 };
        new
    }

    pub fn width(&mut self, val: u32) -> &mut Self {
        let mut new = self;
        if val <= 8 {
            new.wid = 0x0;
        } else if val <= 16 {
            new.wid = 0x8;
        } else if val <= 24 {
            new.wid = 0xF;
        } else if val <= 32 {
            new.wex = 1;
            new.wid = 8;
        }
        new
    }

    pub fn pos(&mut self, val: u32) -> &mut Self {
        let mut new = self;
        new.pos = if val > 1023 { 1023 } else { val };
        new
    }
}

#[derive(Default)]
pub struct Channels {
    ch1: Channel, 
    ch2: Channel
}

#[derive(Default)]
pub struct PCMParams {
    rx: Channels,
    tx: Channels,
    rxon:   u32, //Receieve on.
    txon:   u32, //Transmit on.
    fsm:    u32, //Frame master
    clkm:   u32, //Clock master
    flen:   u32, //Length of frame in clocks.
    fslen:  u32  //Length of first half of frame in clocks.
}

impl PCMParams {
    pub fn rxon(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.rxon = if val { 1 } else { 0 };
        new
    }

    pub fn txon(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.txon = if val { 1 } else { 0 };
        new
    }

    pub fn fs_master(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.fsm = if val { 1 } else { 0 };
        new
    }

    pub fn clk_master(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.clkm = if val { 1 } else { 0 };
        new
    }

    pub fn chlen(&mut self, ch1: u32, ch2: u32) -> &mut Self {
        let mut new = self;
        new.flen = ch1 + ch2;
        new.fslen = ch1;
        new
    }
}

///
/// PCM peripheral registers
///
#[derive(Default)]
pub struct PCM;

impl ops::Deref for PCM {
    type Target = RegisterBlockPCM;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl PCM {
    fn ptr() -> *const RegisterBlockPCM {
        PCM_BASE as *const _
    }

///
///Reference
/// https://github.com/arisena-com/rpi_src/blob/master/apps/i2s_test/src/i2s_test.c
///
    pub fn init(&self, params: &PCMParams) {
//Reset configuration.
        self.CS_A.set(0); //FIXME: Need delay?

//Clear FIFOs and set thresholds. 
        self.CS_A.modify(
            CS_A::RXCLR::SET + //Clear RX FIFO
            CS_A::TXCLR::SET + //Clear TX FIFO
            CS_A::RXTHR::C   + //RXR set when FIFO is less than full.
            CS_A::TXTHR::D     //TXW set when FIFO is one sample shy of full.
        );// FIXME: Need delay?

//Configure receive.
        self.RXC_A.modify (
//Channel 1
            RXC_A::CH1WEX::CLEAR                  + //24bit >= sample size.
            RXC_A::CH1EN.val(params.rx.ch1.en)    + //Enable channel 1.
            RXC_A::CH1POS.val(params.rx.ch1.pos)  + //Channel 1 data position in frame.
            RXC_A::CH1WID.val(params.rx.ch1.wid)  + //Sample width in bits.
//Channel 2
            RXC_A::CH2WEX::CLEAR                  + //24bit >= sample size.
            RXC_A::CH2EN.val(params.rx.ch2.en)    + //Enable channel 2.
            RXC_A::CH2POS.val(params.rx.ch2.pos)  + //Channel 2 data position in frame.
            RXC_A::CH2WID.val(params.rx.ch2.wid)    //Sample width in bits.
        );

//Configure transmit.
        self.TXC_A.modify (
//Channel 1
            TXC_A::CH1WEX::CLEAR                  + //24bit >= sample size.
            TXC_A::CH1EN.val(params.tx.ch1.en)    + //Enable channel 1.
            TXC_A::CH1POS.val(params.tx.ch1.pos)  + //Channel 1 data position in frame.
            TXC_A::CH1WID.val(params.tx.ch1.wid)  + //Sample width in bits.
//Channel 2
            TXC_A::CH2WEX::CLEAR                  + //24bit >= sample size.
            TXC_A::CH2EN.val(params.tx.ch2.en)    + //Enable channel 2.
            TXC_A::CH2POS.val(params.tx.ch2.pos)  + //Channel 2 data position in frame.
            TXC_A::CH2WID.val(params.tx.ch2.wid)    //Sample width in bits.
        );

//Set mode.
        self.MODE_A.modify (
            MODE_A::CLK_DIS::CLEAR          + //Disable PCM clock.
            MODE_A::PDME::PCM               + //Use PCM (standard) input mode.
            MODE_A::FRXP::CLEAR             + //Don't pack 2x16bit samples into one 32bit FIFO location. 
            MODE_A::FTXP::CLEAR             + //Don't pack 2x16bit samples into one 32bit FIFO location. 
            MODE_A::CLKM.val(params.clkm)   + //Clock is an output (master).
            MODE_A::CLKI::CLEAR             + //No clock inversion.
            MODE_A::FSM.val(params.fsm)     + //Frame select is an output (master).
            MODE_A::FSI::CLEAR              + //No frame sync inversion.
            MODE_A::FLEN.val(params.flen)   + //64 clocks in a frame.
            MODE_A::FSLEN.val(params.fslen)   //32 clocks in first half of frame.
        );

//Exit standby.
        self.CS_A.modify(CS_A::STBY::SET);
        //FIXME: DELAY 4 PCM Clocks.

//Enable PCM begin RX & TX.
        self.CS_A.modify (
            CS_A::EN::SET   +
            CS_A::RXON::SET +
            CS_A::TXON::SET
        );
    }
}


/**********************************************************************
 * I2S
 *********************************************************************/

#[derive(Default)]
pub struct I2S;

impl I2S {
    pub fn init(&self, params: &PCMParams) {
        GPFSEL::new().fsel_i2s();
        PCM::default().init(params);
    }
}
