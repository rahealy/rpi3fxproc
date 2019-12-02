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

use super::{SAMPLE_RATE_USIZE, SAMPLE_RATE, SampleType};
use crate::Effect;

///
/// Return fractional part of sample.
///
#[inline]
fn fract(val: SampleType) -> SampleType {
    val - ((val as i32) as SampleType)
}


#[derive(Default)]
pub struct Pwm {
    pub freq:   SampleType,
    pub scale:  SampleType,
    pub offset: SampleType,
    pub duty:   SampleType,
    pub mix:    SampleType,

    cnt:        usize,
}

impl Effect for Pwm {
    fn process(&mut self, _smpl_in: SampleType) -> SampleType {
        self.cnt += 1;
        if self.cnt > SAMPLE_RATE_USIZE {
            self.cnt = 1;
        }
        
        if fract((self.cnt as SampleType) / (SAMPLE_RATE / self.freq)) > self.duty {
            (-1.0 * self.scale + self.offset)
        } else {
            (1.0 * self.scale + self.offset)
        }
    }
    
    fn reset(&mut self) {
        self.cnt    = 1;
        self.freq   = 440.0;
        self.scale  = 1.0;
        self.offset = 0.0;
        self.duty   = 0.50;
        self.mix    = 1.0;
    }

    fn num_params(&mut self) -> usize { 5 }

    fn set_param(&mut self, idx: usize, val: SampleType) {
        match idx {
            0 => { self.freq(val); },
            1 => { self.scale(val); },
            2 => { self.offset(val); },
            3 => { self.duty(val); },
            4 => { self.mix(val); },
            _ => { panic!("Parameter doesn't exist.") }
        }
    }

    fn get_param(&mut self, idx: usize) -> SampleType { 
        match idx {
            0 => self.freq,
            1 => self.scale,
            2 => self.offset,
            3 => self.duty,
            4 => self.mix,
            _ => { panic!("Parameter doesn't exist.") }
        }
    }  

}

impl Pwm {    
    pub fn freq(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.freq = val;
        new
    }

    pub fn scale(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.scale = val;
        new
    }

    pub fn offset(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.offset = val;
        new
    }

    pub fn duty(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.duty = val;
        new
    }

    pub fn mix(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.mix = val;
        new
    }
}
