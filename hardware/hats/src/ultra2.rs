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
 * Reset Pin
 * RPi I/O Pin 29, BCM5,  Function: OUTPUT.
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

use peripherals::{debug, i2c, i2s, timer};
use peripherals::gpset::{GPSET, GPSET0};
use peripherals::gpclr::{GPCLR, GPCLR0};
use peripherals::gpfsel::{GPFSEL};
use drivers::cs4265;
use drivers::cs4265::RegisterAddress;


/**********************************************************************
 * ERROR
 *********************************************************************/

pub enum ERROR {
    I2S(i2s::ERROR),
    CS4265(cs4265::ERROR)
}

impl From<i2s::ERROR> for ERROR {
    fn from(err: i2s::ERROR) -> ERROR {
        ERROR::I2S(err)
    }
}

impl From<cs4265::ERROR> for ERROR {
    fn from(err: cs4265::ERROR) -> ERROR {
        ERROR::CS4265(err)
    }
}

impl ERROR {
    pub fn msg (&self) -> &'static str {
        match self {
            ERROR::I2S(err) => err.msg(),
            ERROR::CS4265(err) => err.msg()
        }
    }
}


/**********************************************************************
 * Params
 *********************************************************************/
#[inline]
fn adc_gain_clip(val: i8) -> i8 {
    return if val < -24 {
        -24
    } else if val > 24 {
        24
    } else {
        val
    }
}

#[derive(Default)]
pub struct Params {
    pdn_mic:  bool,
    pdn_adc:  bool,
    pdn_dac:  bool,
    dacvola:  u8,
    dacvolb:  u8,
    adcgaina: i8,
    adcgainb: i8,
    smplrt:   u32,
    adcsel:   u8,
    mode:     i2s::Mode,
}

impl Params {
///
///Power down/up the microphone.
///
    pub fn pdn_mic(&mut self, pdn: bool) -> &mut Self {
        let mut new = self;
        new.pdn_mic = pdn;
        new
    }


///
///Power down/up the ADC.
///
    pub fn pdn_adc(&mut self, pdn: bool) -> &mut Self {
        let mut new = self;
        new.pdn_adc = pdn;
        new
    }


///
///Power down/up the DAC.
///
    pub fn pdn_dac(&mut self, pdn: bool) -> &mut Self {
        let mut new = self;
        new.pdn_dac = pdn;
        new
    }

///
///Sample rate.
///
    pub fn smplrt(&mut self, val: u32) -> &mut Self {
        let mut new = self;
        new.smplrt = val;
        new
    }


///
///Gain for channel A. Number of half steps from -12dB..12dB.
///
    pub fn adc_gain_a(&mut self, val: i8) -> &mut Self {
        let mut new = self;
        new.adcgaina = adc_gain_clip(val);
        new
    }

///
///Gain for channel B.
///
    pub fn adc_gain_b(&mut self, val: i8) -> &mut Self {
        let mut new = self;
        new.adcgainb = adc_gain_clip(val);
        new
    }

///
///Volume for channel A.
///
    pub fn dac_vol_a(&mut self, val: u8) -> &mut Self {
        let mut new = self;
        new.dacvola = 0xFF - val;
        new
    }

///
///Volume for channel B.
///
    pub fn dac_vol_b(&mut self, val: u8) -> &mut Self {
        let mut new = self;
        new.dacvolb = 0xFF - val;
        new
    }

///
///Source selection for adc. Zero for microphone otherwise line-in.
///
    pub fn adc_sel(&mut self, val: usize) -> &mut Self {
        let mut new = self;
        if val == 0 {
            new.adcsel = cs4265::AICTL::SELECT::MIC.value;
        } else {
            new.adcsel = cs4265::AICTL::SELECT::LINE.value;
        }
        new
    }

///
///Operation mode. Poll, Interrupt, DMA.
///
    pub fn mode(&mut self, mode: i2s::Mode) -> &mut Self {
        let mut new = self;
        new.mode = mode;
        new
    }
}

/**********************************************************************
 * Ultra2
 *********************************************************************/

#[derive(Default)]
pub struct Ultra2<SI2C, SI2S, STIMER> where 
    SI2C: i2c::I2C,
    SI2S: i2s::I2S,
    STIMER: timer::Timer
{
    pub cs4265: cs4265::CS4265<SI2C>,
    pub i2s: SI2S,
    pub timer: STIMER,
}

impl  <II2C, II2S, ITIMER> Ultra2<II2C, II2S, ITIMER> where 
    II2C: i2c::I2C + Default,
    II2S: i2s::I2S + Default,
    ITIMER: timer::Timer + Default
{
    #[inline]
    fn delay_1s(&self) {
        self.timer.one_shot(1_000_000);
    }

    #[inline]
    fn delay_2s(&self) {
        self.timer.one_shot(2_000_000);
    }

///
///The cs4265 uses i2s to communicate audio data to the RPi.
///
    fn cfg_i2s(&self, params: &Params) -> Result<(), ERROR> {
        debug::out("ultra2.cfg_i2s(): Configuring RPi i2s.\r\n");
        let mut pcm = i2s::Params::default();

        pcm.rxon(!params.pdn_adc). //rxon is the opposite of power down
            txon(!params.pdn_dac).
            fs_master(false).
            clk_master(false).
            chlen(32,32).
            smplrt(params.smplrt).
            mode(params.mode);

        pcm.rx.ch1.enable(true).
                   width(32). //Sample width must be 32 bits for i2s.
                   pos(1);    //Sample data starts 1 clock after frame begins.

        pcm.rx.ch2.enable(true).
                   width(32). //Sample width must be 32 bits for i2s.
                   pos(33);   //Data starts 33 clocks after frame begins.

        pcm.tx.ch1.enable(true).
                   width(32). //Sample width must be 32 bits for i2s.
                   pos(1);    //Data starts 1 clock after frame begins.

        pcm.tx.ch2.enable(true).
                   width(32). //Sample width must be 32 bits for i2s.
                   pos(33);   //Data starts 33 clocks after frame begins.

        debug::out("ultra2.cfg_i2s(): Loading i2s configuration.\r\n");
        self.i2s.load(&pcm);
        debug::out("ultra2.cfg_i2s(): i2s configuration loaded.\r\n");

        debug::out("ultra2.cfg_i2s(): RPi i2s configured.\r\n");        
        return Ok(());
    }

    fn cfg_cs4265(&mut self, params: &Params) -> Result<(), ERROR> {
        debug::out("ultra2.cfg_cs4265(): Configuring CS4265.\r\n");

//Power down CS4265.
        debug::out("ultra2.cfg_cs4265(): Powering down CS4265 for configuration.\r\n");
        self.power_down()?;
        debug::out("ultra2.cfg_cs4265(): CS4265 Powered down. Continuing configuration.\r\n");

//Set DAC control.
        self.cs4265.reg.DACCTL1.modify (
            cs4265::DACCTL1::DAC_DIF::I2S24BIT + //Use I2S protocol.
            cs4265::DACCTL1::DEEMPH::CLEAR     + //No de-emphaisis.
            cs4265::DACCTL1::MUTEDAC::CLEAR      //Unmuted.
        );

//Set ADC control.
        self.cs4265.reg.ADCCTL.modify (
            cs4265::ADCCTL::ADC_DIF::I2S24BIT         + //Use I2S protocol.
            cs4265::ADCCTL::HPFFREEZE::CLEAR          + //Leave the dc bias filter unfrozen.
            cs4265::ADCCTL::MUTEADC::CLEAR            + //Unmuted.
            cs4265::ADCCTL::MS::SET                     //Set to master.
        );

//Set clock for sample rate and Ultra2 board's clock rate (12.288 MHz).
        self.cs4265.modify_clk(params.smplrt, 12_288_000)?;

//Set signal selection.
        self.cs4265.reg.SIGSEL.modify (
            cs4265::SIGSEL::SDINSEL::SDIN1 + //Use digital input 1.
            cs4265::SIGSEL::LOOP::CLEAR      //Disable loopback.
        );

//Set gain.
        self.cs4265.reg.PGAA.write(cs4265::PGAA::GAIN.val(params.adcgaina as u8));
        self.cs4265.reg.PGAB.write(cs4265::PGAB::GAIN.val(params.adcgainb as u8));

//Set soft ramp, zero crossing detection and line level.
        self.cs4265.reg.AICTL.modify (
            cs4265::AICTL::PGASOFT::SET + //Use soft ramp on mute and data loss.
            cs4265::AICTL::PGAZERO::SET + //Use zero crossing detection.
            cs4265::AICTL::SELECT.val(params.adcsel) //Select microphone or line.
        );

//Volume.
        self.cs4265.reg.DACVOLA.write(
            cs4265::DACVOLA::VOL.val(params.dacvola)
        );

        self.cs4265.reg.DACVOLB.write(
            cs4265::DACVOLB::VOL.val(params.dacvolb)
        );

//Set soft ramp, zero crossing detection and invert.
        self.cs4265.reg.DACCTL2.modify ( 
            cs4265::DACCTL2::DACSOFT::SET     + //Use soft ramp on mute and data loss.
            cs4265::DACCTL2::DACZERO::SET     + //Use zero crossing detection.
            cs4265::DACCTL2::INVERTDAC::CLEAR   //Do not invert output.
        );

//Select which conditions affect the STATUS register. 
        self.cs4265.reg.STATUSMASK.modify (
            cs4265::STATUSMASK::EFTCM::CLEAR      + //Set status bit on S/PDIF error.
            cs4265::STATUSMASK::CLKERRM::CLEAR    + //Set status bit on clock error.
            cs4265::STATUSMASK::ADCOVFLM::CLEAR   + //Set status bit on overflow error.
            cs4265::STATUSMASK::ADCUNDRFLM::CLEAR   //Set status bit on underflow.
        );

//All updates to status register occur on rising edge.
        self.cs4265.reg.STATUSMODEMSB.modify (
            cs4265::STATUSMODEMSB::EFTC1::CLEAR      +
            cs4265::STATUSMODEMSB::CLKERR1::CLEAR    +
            cs4265::STATUSMODEMSB::ADCOVFL1::CLEAR   +
            cs4265::STATUSMODEMSB::ADCUNDRFL1::CLEAR
        );

//All updates to status register occur on rising edge.
        self.cs4265.reg.STATUSMODELSB.modify (
            cs4265::STATUSMODELSB::EFTC0::CLEAR      +
            cs4265::STATUSMODELSB::CLKERR0::CLEAR    +
            cs4265::STATUSMODELSB::ADCOVFL0::CLEAR   +
            cs4265::STATUSMODELSB::ADCUNDRFL0::CLEAR
        );

//Turn off S/PDIF transmitter.
        self.cs4265.reg.XMITCTL2.modify (
            cs4265::XMITCTL2::TX_DIF::I2S24BIT +
            cs4265::XMITCTL2::TXOFF::SET
        );

//Load configuration.
        debug::out("ultra2.cfg_cs4265(): Loading CS4265 configuration registers.\r\n");
        self.cs4265.ld_regs()?;

//Print status of CS4265.
//         if let Err(err) = self.cs4265.print_status() {
//             return Err(ERROR::CS4265(err));
//         }

//Verify local copy of registers matches CS4265 registers.
//         if let Err(err) = self.cs4265.verify_regs() {
//             return Err(ERROR::CS4265(err));
//         }

        debug::out("ultra2.cfg_cs4265(): CS4265 configured.\r\n");        
        return Ok(());
    }

///
///When reset is on then Ultra2 is put into reset and non-operational.
///
    pub fn reset(&self, on: bool) {
        if on {
            GPCLR::default().GPCLR0.modify(GPCLR0::PCLR5::SET);
            debug::out("ultra2.reset(): Reset enabled.\r\n");
        } else {
            debug::out("ultra2.reset(): Releasing reset. Waiting two seconds for settle.\r\n");
            GPSET::default().GPSET0.modify(GPSET0::PSET5::SET);
            self.delay_2s();
            debug::out("ultra2.reset(): Reset released.\r\n");
        }
    }

///
///Power up the CS4265.
///
    pub fn power_up(&mut self) -> Result<(), ERROR> {
        debug::out("ultra2.power_up(): Powering up.\r\n");
        self.cs4265.reg.POWERCTL.write (
            cs4265::POWERCTL::FREEZE::CLEAR  + //Thaw the registers.
            cs4265::POWERCTL::PDN_MIC::CLEAR + //Power up microphone.
            cs4265::POWERCTL::PDN_ADC::CLEAR + //Power up the ADC
            cs4265::POWERCTL::PDN_DAC::CLEAR + //Power up DAC
            cs4265::POWERCTL::PDN::CLEAR       //Power up device.
        );

        self.cs4265.ld_reg(RegisterAddress::POWERCTL)?;
        self.delay_1s();

//         //Print status of CS4265.
//         if let Err(err) = self.cs4265.print_status() {
//             return Err(ERROR::CS4265(err));
//         }
// 
//         //Verify local copy of registers matches CS4265 registers.
//         if let Err(err) = self.cs4265.verify_regs() {
//             return Err(ERROR::CS4265(err));
//         }

        debug::out("ultra2.power_up(): Powered up.\r\n");
        return Ok(());
    }

///
///Power down the CS4265.
///
    pub fn power_down(&mut self) -> Result<(), ERROR> {
        debug::out("ultra2.power_down(): Powering down.\r\n");
        self.cs4265.reg.POWERCTL.write (
            cs4265::POWERCTL::FREEZE::SET  + //Freeze/thaw the registers.
            cs4265::POWERCTL::PDN_MIC::SET + //Power down microphone.
            cs4265::POWERCTL::PDN_ADC::SET + //Power down the ADC
            cs4265::POWERCTL::PDN_DAC::SET + //Power down DAC
            cs4265::POWERCTL::PDN::SET       //Power down device.
        );

        self.cs4265.ld_reg(RegisterAddress::POWERCTL)?;
        self.delay_1s();

        //Print status of CS4265.
//         if let Err(err) = self.cs4265.print_status() {
//             return Err(ERROR::CS4265(err));
//         }

        //Verify local copy of registers matches CS4265 registers.
//         if let Err(err) = self.cs4265.verify_regs() {
//             return Err(ERROR::CS4265(err));
//         }
        
        debug::out("ultra2.power_down(): Powered down.\r\n");
        return Ok(());
    }

///
///Intialize board.
///
    pub fn init(&mut self) -> Result<(), ERROR> {
        debug::out("Ultra2.init(): Initializing Ultra2\r\n");

//Select the reset pin on the RPi.
        GPFSEL::default().fsel_ultra2();
//Bring the board out of reset.
        self.reset(false);
//Initialize CS4265.
        self.cs4265.init()?;
//Verify local copy of registers matches CS4265 registers.
//         if let Err(err) = self.cs4265.verify_regs() {
//             return Err(ERROR::CS4265(err));
//         }
        debug::out("Ultra2.init(): Ultra2 initialized.\r\n");
        return Ok(());
    }

///
///Load and configure the RPi I2S bus and the CS4265 on the ultra2.
///
///
    pub fn load(&mut self, params: &Params) -> Result<(), ERROR> {
//Configure the RPi i2s bus for communicating with the Ultra2 board.
        self.cfg_i2s(params)?;

//Configure the CS4265.
        self.cfg_cs4265(params)?;

        debug::out("ultra2.cfg(): Ultra2 configured.\r\n");
        return Ok(());
    }
}
