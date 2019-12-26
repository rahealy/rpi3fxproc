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
#![allow(dead_code)]

use super::{Effect, SAMPLE_RATE_USIZE, SAMPLE_RATE, SampleType};
use common::prelude::*;

const PI:                 SampleType = 3.14159265358979;
const QUARTER_TAU:        SampleType = PI / 2.0;
const HALF_TAU:           SampleType = PI;
const THREE_QUARTERS_TAU: SampleType = HALF_TAU + QUARTER_TAU;
const TAU:                SampleType = PI * 2.0;

#[inline]
fn taylor9(x: SampleType) -> SampleType {
    let x2  = x*x;
    let x3  = x2*x;
    let x5  = x3*x2;
    let x7  = x5*x2;
    let x9  = x7*x2;
    x - (x3 / 6.0) + (x5 / 120.0) - (x7 / 5040.0) + (x9 / 362880.0)
}

///
///Given values 0..TAU inclusive return an approximation of sin() using
///taylor series.
///
#[inline]
fn taylor(rad: SampleType) -> SampleType {
    if rad <= HALF_TAU {
        if rad <= QUARTER_TAU {
            taylor9(rad)
        } else {
            taylor9(HALF_TAU - rad)
        }
    } else {
        if rad <= THREE_QUARTERS_TAU {
            -taylor9(rad - HALF_TAU)
        } else {
            -taylor9(TAU - rad)
        }
    }
}

///
/// Return fractional part of sample.
///
#[inline]
fn fract(val: SampleType) -> SampleType {
    val - ((val as i32) as SampleType)
}

#[derive(Default)]
pub struct Sine {
    pub freq:   SampleType,
    pub scale:  SampleType,
    pub offset: SampleType,
    pub mix:    SampleType,

    cnt:        usize,
}

use num_traits::Float;
impl Effect for Sine {

    fn process(&mut self,
               inputs: &[Buffer<SampleType>],
               outputs: &mut [Buffer<SampleType>])
    {
        for i in outputs[0].iter_idx() {
            self.cnt += 1;
            if self.cnt == SAMPLE_RATE_USIZE {
                self.cnt = 0;
            }

            outputs[0].set ( 
                SampleType::sin ( 
                    TAU * fract((self.cnt as SampleType) / (SAMPLE_RATE / self.freq))
                ) * self.scale + self.offset,
                i
            );
        }
    }

    fn reset(&mut self) {
        self.freq   = 440.0;
        self.scale  = 1.0;
        self.offset = 0.0;
        self.mix    = 0.0;
        self.cnt    = 0;
    }
    
    fn num_params(&self) -> usize { 4 }

    fn set_param(&mut self, idx: usize, val: SampleType) {
        match idx {
            0 => { self.freq(val); },
            1 => { self.scale(val); },
            2 => { self.offset(val); },
            3 => { self.mix(val); },
            _ => { panic!("Parameter doesn't exist.") }
        }
    }

    fn get_param(&mut self, idx: usize) -> SampleType { 
        match idx {
            0 => self.freq,
            1 => self.scale,
            2 => self.offset,
            3 => self.mix,
            _ => { panic!("Parameter doesn't exist.") }
        }
    }
}

impl Sine {
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

    pub fn mix(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.mix = val;
        new
    }
}

#[cfg(test)]
mod tests {
    use crate::sine::{Sine, TAU, HALF_TAU, taylor, taylor9, fract};
    use super::{SAMPLE_RATE_USIZE, SAMPLE_RATE, SampleType};

    #[test]
    fn sine() {
        for i in 0..360 {
            let rad = (i as SampleType) * (HALF_TAU / 180.0);
            let t = taylor(rad);
            let t4 = taylor9(rad);
            let s   = SampleType::sin(rad);
            let dif = t - s;
            println!("rad = {}, t = {}, t4 = {}, s = {}", rad, t, t4, s);
//            assert!(SampleType::abs(dif) < 0.000004);
        }

        for cnt in 0..SAMPLE_RATE_USIZE {
            let rad = TAU * fract((cnt as SampleType) / (SAMPLE_RATE / 440.0));
            let t4  = taylor(rad); 
            let s   = SampleType::sin(rad);
            let dif = t4 - s;
            assert!(SampleType::abs(dif) < 0.000004);
        }
    }
}
