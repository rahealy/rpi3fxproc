/*
MIT License

Copyright (c) 2019 Richard A. Healy

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use super::{SAMPLE_RATE, SAMPLE_RATE_USIZE, SampleType};
use common::offset::{Offset};
use crate::Effect;
use peripherals::debug;

const DELAY_SECONDS: usize = 2;
const DELAY_MAX:     usize = (SAMPLE_RATE_USIZE * DELAY_SECONDS);
const DELAY_BUF_SZ:  usize = DELAY_MAX * 2;


/***********************************************************************
 * Delay
 **********************************************************************/

pub struct Delay {
    pub delay:    usize,      //Delay in samples.
    pub feedback: SampleType, //Feedback.
    pub wet:      SampleType, //Wet (delayed) signal.
    pub dry:      SampleType, //Dry (input) signal.

    rd:  Offset,     //Read pointer.
    wr:  Offset,     //Write pointer.
    buf: [SampleType; DELAY_BUF_SZ], //Delay buffer.
}

impl Default for Delay {
    fn default() -> Self {
        Delay {
            delay: DELAY_MAX,
            feedback: 0.25,
            wet: 1.0,
            dry: 0.0,

            rd: Offset(0),
            wr: Offset(DELAY_MAX - 1),
            buf: [SampleType::default(); DELAY_BUF_SZ]
        }
    }
}

impl Effect for Delay {
///
///Process.
///
    fn process(&mut self, smpl_in: SampleType) -> SampleType {
        let smpl_rd  = self.buf[self.rd.0];
        let smpl_wr  = self.buf[self.wr.0];
        let smpl_out = (smpl_in * self.dry) + (smpl_rd * self.wet);

        self.buf[self.wr.0] = smpl_in + (smpl_wr * self.feedback);

        self.wr.inc(DELAY_BUF_SZ);
        self.rd.inc(DELAY_BUF_SZ);

//         debug::out("smpl_in = ");
//         debug::u32hex(smpl_in as u32);
//         debug::out("\r\n");
//         debug::out("smpl_out = ");
//         debug::u32hex(smpl_out as u32);
//         debug::out("\r\n");

        return smpl_out;
    }

///
///Reset delay to defaults.
///
    fn reset(&mut self) {
        self.delay = DELAY_MAX;
        self.feedback = 0.5;
        self.wet = 1.0;
        self.dry = 0.0;

        self.rd = Offset(0);
        self.wr = Offset(DELAY_MAX - 1);
        self.buf = [SampleType::default(); DELAY_BUF_SZ];
    }

    fn num_params(&mut self) -> usize { 4 }

    fn set_param(&mut self, idx: usize, val: SampleType) {
        match idx {
            0 =>  { self.delay(val); },
            1 =>  { self.feedback(val); },
            2 =>  { self.wet(val); },
            3 =>  { self.dry(val); },
            _ => { panic!("Parameter doesn't exist.") }
        }
    }

    fn get_param(&mut self, idx: usize) -> SampleType { 
        match idx {
            0 =>  self.delay as SampleType,
            1 =>  self.feedback,
            2 =>  self.wet,
            3 =>  self.dry,
            _ => { panic!("Parameter doesn't exist.") }
        }
    }
}

impl Delay {
///
///Write position is always "delay" samples ahead of read.
///
    #[inline]
    fn update_wr_pos(&mut self) {
        self.wr = *Offset(self.rd.val()).add(self.delay).wrap(DELAY_BUF_SZ);
    }

///
///Set delay in seconds. Delay is rounded to the nearest whole 
///sample divisible by two.
///
    pub fn delay(&mut self, sec: SampleType) -> &mut Self {
        let mut new = self;

//Must be an integer multiple of 2.
        let delay = ((((sec * SAMPLE_RATE) + 0.5) as usize) / 2) * 2;
        if delay > DELAY_MAX {
            new.delay = DELAY_MAX;
        } else {
            new.delay = delay;
        }

        new.update_wr_pos();
        
        new
    }

///
///Feedback in percent.
///
    pub fn feedback(&mut self, per: SampleType) -> &mut Self {
        let mut new = self;
        new.feedback = per;
        new
    }

///
///wet signal in percent.
///
    pub fn wet(&mut self, per: SampleType) -> &mut Self {
        let mut new = self;
        new.wet = per;
        new
    }

///
///dry signal in percent.
///
    pub fn dry(&mut self, per: SampleType) -> &mut Self {
        let mut new = self;
        new.dry = per;
        new
    }
}
