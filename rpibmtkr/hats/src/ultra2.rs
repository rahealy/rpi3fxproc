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
 * I2S Pins
 * RPi I/O Pin 12, BCM18, Function: PCM_CLK, I2S: BCLK
 * RPi I/O Pin 35, BCM19, Function: PCM_FS, I2S: LRCLK
 * RPi I/O Pin 38, BCM20, Function: PCM_DIN, I2S: SDIN
 * RPi I/O Pin 40, BCM21, Function: PCM_DOUT, I2S: SOUT
 */

/*
 * Clock table for CS4265
 * 
 * Given Ultra2 MCLK of 12288000 Hz:
 *
 * Single Speed (ADCCTL::FM::SINGLE_SPEED_4_50_KHZ)
 * Fs      MCLK  Divider
 * ------------------------------------------------
 * 32kHz,  384x, 1.5 (MCLK::DIV::DIV1_5)
 * 48kHz,  256x, 1.0 (MCLK::DIV::DIV1_0)
 * 
 * Double Speed (ADCCTL::FM::DOUBLE_SPEED_50_100_KHZ)
 * --------------------------------------------------
 * 64kHz,  192x, 1.5 (MCLK::DIV::DIV1_5)
 * 96kHz,  128x, 1.0 (MCLK::DIV::DIV1_0)
 * 
 * Quad Speed (ADCCTL::FM::QUAD_SPEED_100_200_KHZ)
 * -----------------------------------------------
 * 128kHz, 96x, 1.5 (MCLK::DIV::DIV1_5)
 * 192kHz, 64x, 1.0 (MCLK::DIV::DIV1_0)
 *
 */

use peripherals::MMIO_BASE;
use peripherals::{debug, i2c, i2s, timer};
use drivers::cs4265;
use register::register_bitfields;
use register::mmio::ReadWrite;


/**********************************************************************
 * ERROR
 *********************************************************************/

pub enum ERROR {
    I2C(i2c::ERROR),
    I2S(i2s::ERROR),
    CS4265(cs4265::ERROR)
}

impl ERROR {
    pub fn msg (&self) -> &'static str {
        match self {
            ERROR::I2C(err) => err.msg(),
            ERROR::I2S(err) => err.msg(),
            ERROR::CS4265(err) => err.msg()
        }
    }
}


/**********************************************************************
 * GPIO
 *********************************************************************/

register_bitfields! {
    u32,

/// GPIO Function Select 0
    GPFSEL0 [
/// RPi I/O Pin 29 (BCM5) used as reset for the ultra2.
        FSEL5 OFFSET(15) NUMBITS(3) [
            OUTPUT = 0b001
        ]
    ],

///GPIO Set pin.
    GPSET0 [
///Set RPi I/O Pin 29 (BCM5) to bring ultra2 out of reset condition.
        PSET5 OFFSET(5) NUMBITS(1) []
    ],

///GPIO Clear pin.
    GPCLR0 [
///Clear RPi Pin 29 (BCM5) to put ultra2 into reset condition.
        PCLR5 OFFSET(5) NUMBITS(1) []
    ]
}


///
///GPFSEL2 alternative function select register - 0x7E200008
///
const GPFSEL0_OFFSET: u32 = 0x0020_0000;
const GPFSEL0_BASE:   u32 = MMIO_BASE + GPFSEL0_OFFSET;

///
///Function select register for the GPIO pin used by the Ultra2 board.
///
pub const GPFSEL0: *const ReadWrite<u32, GPFSEL0::Register> =
    GPFSEL0_BASE as *const ReadWrite<u32, GPFSEL0::Register>;


///
///GPSET0 pin set register - 0x7E20001C
///
const GPSET0_OFFSET: u32 = 0x0020_001C;
const GPSET0_BASE:   u32 = MMIO_BASE + GPSET0_OFFSET;

///
///Output set register for the GPIO pin used by the Ultra2 board.
///
pub const GPSET0: *const ReadWrite<u32, GPSET0::Register> =
    GPSET0_BASE as *const ReadWrite<u32, GPSET0::Register>;


///
///GPCLR0 pin set register - 0x7E200028
///
const GPCLR0_OFFSET: u32 = 0x0020_0028;
const GPCLR0_BASE:   u32 = MMIO_BASE + GPCLR0_OFFSET;

///
///Output clear register for the GPIO pin used by the Ultra2 board.
///
pub const GPCLR0: *const ReadWrite<u32, GPCLR0::Register> =
    GPCLR0_BASE as *const ReadWrite<u32, GPCLR0::Register>;


/**********************************************************************
 * Ultra2
 *********************************************************************/

pub struct Ultra2<SI2C, SI2S, STIMER> where 
    SI2C: i2c::I2C,
    SI2S: i2s::I2S,
    STIMER: timer::Timer
{
    pub cs4265: cs4265::CS4265<SI2C>,
    pub i2s: SI2S,
    pub timer: STIMER
}


impl <II2C, II2S, ITIMER> Default for Ultra2<II2C, II2S, ITIMER> where 
    II2C: i2c::I2C + Default,
    II2S: i2s::I2S + Default,
    ITIMER: timer::Timer + Default
{
    fn default() -> Ultra2<II2C, II2S, ITIMER> {
        Ultra2::<II2C, II2S, ITIMER> {
            cs4265: cs4265::CS4265::<II2C>::default(),
            i2s: <II2S>::default(),
            timer: <ITIMER>::default()
        }
    }
}

impl  <II2C, II2S, ITIMER> Ultra2<II2C, II2S, ITIMER> where 
    II2C: i2c::I2C + Default,
    II2S: i2s::I2S + Default,
    ITIMER: timer::Timer + Default
{

///
/// Bring ultra2 out of reset. Poll for condition of CS4265 SDOUT pin 
/// and save i2c address for use in further accesses.
///
    pub fn init(&mut self) -> Result<(), ERROR> {
        debug::out("ultra2.init(): Releasing reset. Waiting two seconds for settle.\r\n");
        unsafe {
            (*GPFSEL0).modify(GPFSEL0::FSEL5::OUTPUT);
            (*GPSET0).modify(GPSET0::PSET5::SET);
        }
        self.timer.one_shot(2_000_000);

//Initialize CS4265.
        if let Err(err) = self.cs4265.init() {
            match err {
                cs4265::ERROR::I2C(e) => {
                    return Err(ERROR::I2C(e));
                },
                _ => {
                    return Err(ERROR::CS4265(err));
                }
            }
        }

//Configure CS4265
        debug::out("ultra2.init(): Configuring CS4265.\r\n");
        
//Set power control register.
        self.cs4265.reg.POWERCTL.write (
            cs4265::POWERCTL::FREEZE::SET    + //Freeze the registers.
            cs4265::POWERCTL::PDN_MIC::SET   + //Power down the microphone.
            cs4265::POWERCTL::PDN_ADC::CLEAR + //Power up the ADC
            cs4265::POWERCTL::PDN_DAC::CLEAR + //Power up the DAC
            cs4265::POWERCTL::PDN::SET         //Power down the whole device until load.
        );

//Set DAC control.
        self.cs4265.reg.DACCTL1.write (
            cs4265::DACCTL1::DAC_DIF::I2S24BIT + //Use I2S protocol.
            cs4265::DACCTL1::DEEMPH::CLEAR     + //No de-emphaisis.
            cs4265::DACCTL1::MUTEDAC::CLEAR      //Unmuted.
        );

//Set ADC control and MCLK for 48kHz sample rate (see table in comment at top of this file.)
        self.cs4265.reg.ADCCTL.write (
            cs4265::ADCCTL::FM::SINGLE_SPEED_4_50_KHZ + //Use single speed per table.
            cs4265::ADCCTL::ADC_DIF::I2S24BIT         + //Use I2S protocol.
            cs4265::ADCCTL::HPFFREEZE::CLEAR          + //Leave the dc bias filter unfrozen.
            cs4265::ADCCTL::MUTEADC::CLEAR            + //Unmuted.
            cs4265::ADCCTL::MS::SET                     //Set to master.
        );

//Set MCLK divider.
        self.cs4265.reg.MCLK.write (
            cs4265::MCLK::DIV::DIV1_0  //Divider is 1.0 per table.
        );

//Set signal selection.
        self.cs4265.reg.SIGSEL.write (
            cs4265::SIGSEL::SDINSEL::SDIN1 + //Use digital input 1.
            cs4265::SIGSEL::LOOP::CLEAR      //Disable loopback.
        );

//Set gain to 0 dB.
        self.cs4265.reg.PGAB.set(0);
        self.cs4265.reg.PGAA.set(0);

//Set soft ramp, zero crossing detection and line level.
        self.cs4265.reg.AICTL.write (
            cs4265::AICTL::PGASOFT::SET + //Use soft ramp on mute and data loss.
            cs4265::AICTL::PGAZERO::SET + //Use zero crossing detection.
            cs4265::AICTL::SELECT::LINE   //Line level.
        );

//Full volume.
        self.cs4265.reg.DACVOLA.set(0);
        self.cs4265.reg.DACVOLB.set(0);

//Set soft ramp, zero crossing detection and invert.
        self.cs4265.reg.DACCTL2.write (
            cs4265::DACCTL2::DACSOFT::SET     + //Use soft ramp on mute and data loss.
            cs4265::DACCTL2::DACZERO::SET     + //Use zero crossing detection.
            cs4265::DACCTL2::INVERTDAC::CLEAR   //Do not invert output.
        );
        
//Select which conditions affect the STATUS register. 
        self.cs4265.reg.STATUSMASK.write (
            cs4265::STATUSMASK::EFTCM::CLEAR    + //Not using S/PDIF.
            cs4265::STATUSMASK::CLKERRM::SET    + //Set status bit on clock error.
            cs4265::STATUSMASK::ADCOVFLM::SET   + //Set status bit on overflow error.
            cs4265::STATUSMASK::ADCUNDRFLM::SET   //Set status bit on underflow.
        );

//All updates to status register occur on rising edge.
        self.cs4265.reg.STATUSMODEMSB.write (
            cs4265::STATUSMODEMSB::EFTC1::CLEAR      +
            cs4265::STATUSMODEMSB::CLKERR1::CLEAR    +
            cs4265::STATUSMODEMSB::ADCOVFL1::CLEAR   +
            cs4265::STATUSMODEMSB::ADCUNDRFL1::CLEAR
        );

//All updates to status register occur on rising edge.
        self.cs4265.reg.STATUSMODELSB.write (
            cs4265::STATUSMODELSB::EFTC0::CLEAR      +
            cs4265::STATUSMODELSB::CLKERR0::CLEAR    +
            cs4265::STATUSMODELSB::ADCOVFL0::CLEAR   +
            cs4265::STATUSMODELSB::ADCUNDRFL0::CLEAR
        );

//Turn off S/PDIF transmitter.
        self.cs4265.reg.XMITCTL2.write( 
            cs4265::XMITCTL2::TXOFF::SET
        );

//Load configuration.
        if let Err(err) = self.cs4265.load() {
            return Err(ERROR::CS4265(err));
        }

        debug::out("ultra2.init(): CS4265 configured. Waiting power up.\r\n");

//The cs4265 uses i2s to communicate audio data to the RPi.
//Set up RPi i2s as slave for 2 channel 48kHz, 16bit audio. 
        let mut pcm = i2s::PCMParams::default();

        pcm.rxon(true).
            txon(true).
            fs_master(false).
            clk_master(false).
            chlen(32,32);  //CS4265 has a 64 bit frame length.

        pcm.rx.ch1.enable(true).
                   width(24). //Sample width is 24 bits.
                   pos(1);    //Sample data starts 1 clock after frame begins.

        pcm.rx.ch2.enable(true).
                   width(24). //Sample width is 24 bits.
                   pos(33);   //Data starts 33 clocks after frame begins.

        pcm.tx.ch1.enable(true).
                   width(24). //Sample width is 24 bits.
                   pos(1);    //Data starts 1 clock after frame begins.

        pcm.tx.ch2.enable(true).
                   width(24). //Sample width is 24 bits.
                   pos(33);   //Data starts 33 clocks after frame begins.

        self.i2s.load(&pcm);

        return Ok(());
    }
}
