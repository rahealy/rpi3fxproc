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
use crate::debug;
use crate::timer::{Timer, Timer1};
use crate::gpfsel::GPFSEL;
use crate::clk::PCMCTL;


/**********************************************************************
 * ERROR
 *********************************************************************/

///
///Module error definitions.
///
pub enum ERROR {
///
    TIMEOUT,
    SYNC,
    FLOW
}

impl ERROR {
    pub fn msg (&self) -> &'static str {
        match self {
            ERROR::TIMEOUT   => "Unexpected timeout.",
            ERROR::SYNC      => "Frame is out of sync.",
            ERROR::FLOW      => "Over/under flow condition."
        }
    }
}

/**********************************************************************
 * I2S
 *********************************************************************/

pub trait I2S {
    fn init() {
        debug::out("i2s.init(): Initializing I2S.\r\n");
        GPFSEL::default().fsel_i2s();  //Select the GPIO pins for I2S.
        debug::out("i2s.init(): I2S initialized.\r\n");
    }

    fn load(&self, params: &Params);
//    fn print_status(&self);
    fn tx_on(&self, val: bool);
    fn rx_on(&self, val: bool);
}

/**********************************************************************
 * Params
 *********************************************************************/

#[derive(Default)]
pub struct Channel {
    en:  bool, //Channel enable.
    wid: u32,  //Bit depth.
    wex: u32,  //Bit extend for >24 bit samples.
    pos: u32,  //Position in frame.
}

impl Channel {
    pub fn enable(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.en = val;
        new
    }

    pub fn width(&mut self, val: u32) -> &mut Self {
        let mut new = self;
        new.wex = 0;
        if val <= 8 {         //8 bit.
            new.wid = 0x0; 
        } else if val <= 16 { //16 bit.
            new.wid = 0x8;
        } else if val <= 24 { //24 bit.
            new.wid = 0xF;
        } else {              //32 bit.
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
    pub ch1: Channel, 
    pub ch2: Channel
}

impl Channels {
    fn nchans(&self) -> u32 {
        self.ch1.en as u32 + self.ch2.en as u32
    }

    fn nbits(&self) -> u32 {
        let wid1: u32 = if self.ch1.en { self.ch1.wid } else { 0 };
        let wid2: u32 = if self.ch2.en { self.ch2.wid } else { 0 };
        if wid1 > wid2 { wid1 } else { wid2 }
    }
}

#[derive(Default)]
pub struct Params {
    pub rx: Channels,
    pub tx: Channels,
    rxon:   bool, //Receieve on.
    txon:   bool, //Transmit on.
    fsm:    bool, //Frame master
    clkm:   bool, //Clock master
    flen:   u32,  //Length of frame in clocks.
    fslen:  u32,  //Length of first half of frame in clocks.
    smplrt: u32   //Sample rate in samples per second.
}

impl Params {
    pub fn rxon(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.rxon = val;
        new
    }

    pub fn txon(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.txon = val;
        new
    }

    pub fn fs_master(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.fsm = val;
        new
    }

    pub fn clk_master(&mut self, val: bool) -> &mut Self {
        let mut new = self;
        new.clkm = val;
        new
    }

    pub fn chlen(&mut self, ch1: u32, ch2: u32) -> &mut Self {
        let mut new = self;
        new.flen = ch1 + ch2;
        new.fslen = ch1;
        new
    }
    
    pub fn smplrt(&mut self, smplrt: u32) -> &mut Self {
        let mut new = self;
        new.smplrt = smplrt;
        new
    }
    
    fn nchans(&self) -> u32 {
        let rx = self.rx.nchans();
        let tx = self.tx.nchans();
        if rx > tx { rx } else { tx }
    }

    fn nbits(&self) -> u32 {
        let rx = self.rx.nbits();
        let tx = self.tx.nbits();
        if rx > tx { rx } else { tx }
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
            A  = 0b00, //FIXME: BCM Documentation unclear. Under 02 samples.
            B  = 0b01, //FIXME: BCM Documentation unclear. Under 17 samples.
            C  = 0b10, //FIXME: BCM Documentation unclear. Under 49 samples.
            D  = 0b11  //FIXME: BCM Documentation unclear. Under 64 samples.
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
        EN OFFSET(0) NUMBITS(1) []
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
        PDME OFFSET(26) NUMBITS(1) [],

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
    INTSTC_A [
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
    pub CS_A:       ReadWrite<u32, CS_A::Register>,

///FIFO Buffer. Offset 0x4.
    pub FIFO_A:     ReadWrite<u32, FIFO_A::Register>,

///Mode. Offset 0x8.
    pub MODE_A:     ReadWrite<u32, MODE_A::Register>,

///Receive configuration. Offset 0xC.
    pub RXC_A:      ReadWrite<u32, RXC_A::Register>,

///Transmit configuration. Offset 0x10.
    pub TXC_A:      ReadWrite<u32, TXC_A::Register>,

///DMA Request level. Offset 0x14.
    pub DREQ_A:     ReadWrite<u32, DREQ_A::Register>,

///Interupt enable. Offset 0x18.
    pub INTEN_A:    ReadWrite<u32, INTEN_A::Register>,

///Interrupt status and clear. Offset 0x1C.
    pub INTSTC_A:   ReadWrite<u32, INTSTC_A::Register>,

///Gray mode control. Offset 0x20.
    pub GRAY:       ReadWrite<u32, GRAY::Register>
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

impl I2S for PCM {
///
///Load the provided configuration.
///
/// Sets and enables the PCM clock.
/// Sets and enables the PCM module.
/// Both tx and rx are off and must be turned on with calls to [tx,rx]_on(true). 
///
///Reference
/// https://github.com/arisena-com/rpi_src/blob/master/apps/i2s_test/src/i2s_test.c
///
    fn load(&self, params: &Params) {
        debug::out("pcm.load(): Loading parameters.\r\n");

//Disable PCM and clear CS_A register.
        debug::out("pcm.load(): Disable PCM.\r\n");
        self.CS_A.set(0);
        PCM::wait_1sec();
        debug::out("pcm.load(): PCM disabled.\r\n");
        
//Set clock speed.
        if params.clkm | params.fsm {
            debug::out("pcm.load(): Master. Set PCM clock speed.\r\n");
            PCMCTL::default().i2s_enable (params.smplrt * params.nbits() * params.nchans());
        } else {
            debug::out("pcm.load(): Slave. No clock necessary.\r\n");
            PCMCTL::default().i2s_disable();
        }

//Enable PCM.
        debug::out("pcm.load(): Enable PCM.\r\n");
        self.CS_A.write( CS_A::EN::SET );
        PCM::wait_1sec();
        debug::out("pcm.load(): PCM enabled.\r\n");

//Configure receive.
        self.RXC_A.modify (
//Channel 1
//            RXC_A::CH1WEX.val(params.rx.ch1.wex)      + //24bit >= sample size.
            RXC_A::CH1WEX::CLEAR      + //24bit >= sample size.
//            RXC_A::CH1EN.val(params.rx.ch1.en as u32) + //Enable channel 1.
            RXC_A::CH1EN::SET + //Enable channel 1.
            RXC_A::CH1POS.val(params.rx.ch1.pos)      + //0 based index data position in frame.
//            RXC_A::CH1WID.val(params.rx.ch1.wid)      + //Sample width in bits.
            RXC_A::CH1WID.val(0xF) +
//Channel 2
//            RXC_A::CH2WEX.val(params.rx.ch1.wex)      + //24bit >= sample size.
            RXC_A::CH2WEX::CLEAR      + //24bit >= sample size.
            RXC_A::CH2EN::SET + //Enable channel 2.
//            RXC_A::CH2EN.val(params.rx.ch2.en as u32) + //Enable channel 2.
            RXC_A::CH2POS.val(params.rx.ch2.pos)      + //0 based index data position in frame.
//            RXC_A::CH2WID.val(params.rx.ch2.wid)        //Sample width in bits.
            RXC_A::CH2WID.val(0xF)        //Sample width in bits.
        );

//Configure transmit.
        self.TXC_A.modify (
//Channel 1
//            TXC_A::CH1WEX.val(params.tx.ch1.wex)      + //24bit >= sample size.
            TXC_A::CH1WEX::CLEAR      + //24bit >= sample size.
//            TXC_A::CH1EN.val(params.tx.ch1.en as u32) + //Enable channel 1.
            TXC_A::CH1EN::SET + //Enable channel 1.
            TXC_A::CH1POS.val(params.tx.ch1.pos)      + //0 based index data position in frame.
//            TXC_A::CH1WID.val(params.tx.ch1.wid)      + //Sample width in bits.
            TXC_A::CH1WID.val(0xF)      + //Sample width in bits.
//Channel 2
//            TXC_A::CH2WEX.val(params.tx.ch2.wex)      + //24bit >= sample size.
            TXC_A::CH2WEX::CLEAR      + //24bit >= sample size.
//            TXC_A::CH2EN.val(params.tx.ch2.en as u32) + //Enable channel 2.
            TXC_A::CH2EN::SET + //Enable channel 1.
            TXC_A::CH2POS.val(params.tx.ch2.pos)      + //0 based index data position in frame.
//            TXC_A::CH2WID.val(params.tx.ch2.wid)        //Sample width in bits.
            TXC_A::CH2WID.val(0xF)      //Sample width in bits.
        );

//Set mode. 
        self.MODE_A.modify (
            MODE_A::CLK_DIS::CLEAR                  + //Enable PCM clock.
            MODE_A::PDME::CLEAR                     + //PDM is for digital microphones. Disable.
            MODE_A::FRXP::CLEAR                     + //Don't pack 2x16bit samples into one 32bit FIFO location. 
            MODE_A::FTXP::CLEAR                     + //Don't pack 2x16bit samples into one 32bit FIFO location. 
            MODE_A::CLKM.val(!params.clkm as u32)   + //Clock is an output (master) or input (slave).
            MODE_A::CLKI::CLEAR                     + //No clock inversion.
            MODE_A::FSM.val(!params.fsm as u32)     + //Frame select is an output (master) or input (slave).
            MODE_A::FSI::CLEAR                      + //No frame sync inversion.
            MODE_A::FLEN.val(params.flen)           + //Clocks in a L/R frame.
            MODE_A::FSLEN.val(params.fslen)           //Clocks in first half of frame.
        );

//Clear FIFOs. 
        debug::out("pcm.load(): Clear FIFOs.\r\n");
        self.CS_A.modify (
            CS_A::RXCLR::SET + //Clear RX FIFO
            CS_A::TXCLR::SET   //Clear TX FIFO
        );
        PCM::wait_1sec();

//Set thresholds.
        debug::out("pcm.load(): Set FIFO thresholds.\r\n");
        self.CS_A.modify (
            CS_A::RXTHR::D   + //RXR set when FIFO is less than full.
            CS_A::TXTHR::D     //TXW set when FIFO is one sample shy of full.
        );
        PCM::wait_1sec();

//Exit standby.
        debug::out("pcm.load(): Exit RAM standby.\r\n");
        self.CS_A.modify(CS_A::STBY::SET);
        PCM::wait_1sec();

        debug::out("pcm.load(): Parameters loaded.\r\n");
    }

    fn tx_on(&self, val: bool) {
        if val {
            debug::out("pcm.tx_on(): Enabling PCM transmit (TX).\r\n");
            self.CS_A.modify (CS_A::TXON::SET);
        } else {
            debug::out("pcm.tx_on(): Disabling PCM transmit (TX).\r\n");
            self.CS_A.modify (CS_A::TXON::CLEAR);
        }
        PCM::wait_1sec();
    }

    fn rx_on(&self, val: bool) {
        if val {
            debug::out("pcm.enable_rx(): Enabling PCM receive (RX).\r\n");
            self.CS_A.modify (CS_A::RXON::SET);
        } else {
            debug::out("pcm.enable_rx(): Disabling PCM receive (RX).\r\n");
            self.CS_A.modify (CS_A::RXON::CLEAR);
        }
    }
}

impl PCM {
    fn ptr() -> *const RegisterBlockPCM {
        PCM_BASE as *const _
    }

///
///PCM provides a SYNC bit that echoes back the written value after 2 clocks.
///Return after clks / 2 periods have elapsed.
///FIXME: This doesn't work right. Don't use.
///
    pub fn sync(&self, clks: usize) {
        debug::out("pcm.sync(): Start sync.\r\n");
        for _ in 0..(clks / 2) {
            self.CS_A.modify (CS_A::SYNC::SET);
            while !self.CS_A.is_set(CS_A::SYNC) {}
        }
        debug::out("pcm.sync(): End sync.\r\n");
    }

    fn wait_1sec() {
        Timer1::default().one_shot(1_000_000);
    }


    fn poll_rx_error(&self) -> Result<(), ERROR> {
        let cs = self.CS_A.extract();

//Under or overflow error.
        if cs.is_set(CS_A::RXERR) {
            return Err(ERROR::FLOW);
        }

//FIFO is in sync with data frame.
        if cs.is_set(CS_A::RXSYNC) {
            return Err(ERROR::SYNC);
        }

        return Ok(());
    }

///
///Write the interupt status register contents to debug.
///
    pub fn print_int_status(&self) {
        let intstc = self.INTSTC_A.extract();
        debug::out("          10987654321098765432109876543210\r\n");
        debug::out("INTSTC_A: ");
        debug::u32bits(intstc.get());
        debug::out("\r\n");

        debug::out( if intstc.is_set(INTSTC_A::RXR)    { "IRXR = 1\r\n" } else { "IRXR = 0\r\n" } );
        debug::out( if intstc.is_set(INTSTC_A::TXW)    { "ITXW = 1\r\n" } else { "ITXW = 0\r\n" } );
        debug::out( if intstc.is_set(INTSTC_A::RXERR)  { "IRXERR = 1\r\n" } else { "IRXERR = 0\r\n" } );
        debug::out( if intstc.is_set(INTSTC_A::TXERR)  { "ITXERR = 1\r\n" } else { "ITXERR = 0\r\n" } );
    }

///
///Write the status bits to debug.
///
    pub fn print_status(&self) {
        let cs = self.CS_A.extract();

        debug::out("      10987654321098765432109876543210\r\n");
        debug::out("CS_A: ");
        debug::u32bits(cs.get());
        debug::out("\r\n");

        debug::out( if cs.is_set(CS_A::RXF)    { "RXF = 1\r\n" } else { "RXF = 0\r\n" } );
        debug::out( if cs.is_set(CS_A::TXE)    { "TXE = 1\r\n" } else { "TXE = 0\r\n" } );
        debug::out( if cs.is_set(CS_A::RXD)    { "RXD = 1\r\n" } else { "RXD = 0\r\n" } );
        debug::out( if cs.is_set(CS_A::TXD)    { "TXD = 1\r\n" } else { "TXD = 0\r\n" } );
        debug::out( if cs.is_set(CS_A::RXR)    { "RXR = 1\r\n" } else { "RXR = 0\r\n" } );
        debug::out( if cs.is_set(CS_A::TXW)    { "TXW = 1\r\n" } else { "TXW = 0\r\n" } );
        debug::out( if cs.is_set(CS_A::RXERR)  { "RXERR = 1\r\n" } else { "RXERR = 0\r\n" } );
        debug::out( if cs.is_set(CS_A::TXERR)  { "TXERR = 1\r\n" } else { "TXERR = 0\r\n" } );
        debug::out( if cs.is_set(CS_A::RXSYNC) { "RXSYNC = 1\r\n" } else { "RXSYNC = 0\r\n" } );
        debug::out( if cs.is_set(CS_A::TXSYNC) { "TXSYNC = 1\r\n" } else { "TXSYNC = 0\r\n" } );
    }

///
///Write value to FIFO until FIFO is full.
///
///Reference:
/// https://github.com/arisena-com/rpi_src/blob/master/apps/i2s_test/src/i2s_test.c
///
    pub fn tx_fill(&self, val: u32) {
        debug::out("pcm.tx_fill(): Begin.\r\n");
        let mut i: u32 = 0;
        while self.CS_A.is_set(CS_A::TXW) {
            if self.CS_A.is_set(CS_A::TXERR) {
                self.CS_A.modify(CS_A::TXERR::SET);
            }
            self.FIFO_A.write ( FIFO_A::DATA.val(val) );
            i += 1;
            if i > 1000 {
                debug::out("pcm.tx_fill(): Timeout.\r\n");
                return;
            }
        }
        debug::u32bits(i);
        debug::out("\r\n");
        debug::out("pcm.tx_fill(): End.\r\n");
    }

///
///Write value to FIFO num times.
///
    pub fn tx_write_val(&self, val: u32, num: usize) -> u32 {
//        debug::out("pcm.write_val(): Begin.\r\n");
        if num > 0 {
            let mut e = 0;
            let mut i = num;
//            self.print_status();

            loop {
                while self.CS_A.is_set(CS_A::TXD) {
                    if self.CS_A.is_set(CS_A::TXERR) {
                        e += 1;
                        self.CS_A.modify(CS_A::TXERR::SET);
                    }

                    self.FIFO_A.write ( FIFO_A::DATA.val(val) );
                    i -= 1;
                    if i > 0 {
                        continue;
                    } else {
                        return e;
                    }
                }
            }
        }

        return 0xFFFFFFFF;
    }
}

/**********************************************************************
 * I2S0
 *********************************************************************/

pub type I2S0 = PCM;
