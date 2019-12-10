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

use super::{SampleType, SAMPLE_RATE};
use crate::Effect;
use biquad::*;

const LOW_PASS_FREQ: f64 = 100.0;
const HIGH_PASS_FREQ: f64 = 500.0;

pub struct LowPass (DirectForm2Transposed<SampleType>);
impl Default for LowPass {
    fn default() -> Self {
        if let Ok(lp) = Coefficients::<SampleType>::from_params ( 
            Type::LowPass, 
            (SAMPLE_RATE as f32).hz(),
            LOW_PASS_FREQ.hz(),
            Q_BUTTERWORTH_F64
        ) {
            LowPass(DirectForm2Transposed::<SampleType>::new(lp))
        } else {
            panic!("effects::tone_lmh::LowPass() unable to create coefficients.");
        }
    }
}

pub struct HighPass (DirectForm2Transposed<SampleType>);
impl Default for HighPass {
    fn default() -> Self {
        if let Ok(lp) = Coefficients::<SampleType>::from_params ( 
            Type::HighPass, 
            (SAMPLE_RATE as f32).hz(),
            HIGH_PASS_FREQ.hz(),
            Q_BUTTERWORTH_F64
        ) {
            HighPass(DirectForm2Transposed::<SampleType>::new(lp))
        } else {
            panic!("effects::tone_lmh::HighPass() unable to create coefficients.");
        }
    }
}

pub struct MidLowPass (DirectForm2Transposed<SampleType>);
impl Default for MidLowPass {
    fn default() -> Self {
        if let Ok(lp) = Coefficients::<SampleType>::from_params ( 
            Type::HighPass, 
            (SAMPLE_RATE as f32).hz(),
            HIGH_PASS_FREQ.hz(),
            Q_BUTTERWORTH_F64
        ) {
            MidLowPass(DirectForm2Transposed::<SampleType>::new(lp))
        } else {
            panic!("effects::tone_lmh::MidLowPass() unable to create coefficients.");
        }
    }
}

pub struct MidHighPass (DirectForm2Transposed<SampleType>);
impl Default for MidHighPass {
    fn default() -> Self {
        if let Ok(lp) = Coefficients::<SampleType>::from_params ( 
            Type::HighPass, 
            (SAMPLE_RATE as f32).hz(),
            LOW_PASS_FREQ.hz(),
            Q_BUTTERWORTH_F64
        ) {
            MidHighPass(DirectForm2Transposed::<SampleType>::new(lp))
        } else {
            panic!("effects::tone_lmh::MidHighPass() unable to create coefficients.");
        }
    }
}

/***********************************************************************
 * ToneLMH
 **********************************************************************/

pub struct ToneLMH {
    pub low:  SampleType,  //Scale low pass filter output.
    pub mid:  SampleType,  //Scale of mid pass notch filter output.
    pub high: SampleType, //Scale of high pass filter output.
    pub lp:   LowPass,
    pub hp:   HighPass,
    pub mlp:  MidLowPass,
    pub mhp:  MidHighPass,
}


impl Default for ToneLMH {
    fn default() -> Self {
        ToneLMH {
            low: 1.0,
            mid: 1.0,
            high: 1.0,
            lp: LowPass::default(),
            hp: HighPass::default(),
            mlp: MidLowPass::default(),
            mhp: MidHighPass::default(),
        }
    }
}

impl Effect for ToneLMH {
///
///Process.
///
    fn process(&mut self, smpl_in: SampleType) -> SampleType {
        smpl_in + (self.lp.0.run(smpl_in) * self.low)  + 
                  (self.hp.0.run(smpl_in) * self.high) + 
                  (self.mlp.0.run(smpl_in) * self.mid) +
                  (self.mhp.0.run(smpl_in) * self.mid)
    }

///
///Reset to defaults.
///
    fn reset(&mut self) {
        self.low = 1.0;
        self.mid = 1.0;
        self.high = 1.0;
    }

    fn num_params(&self) -> usize { 3 }

    fn set_param(&mut self, idx: usize, val: SampleType) {
        match idx {
            0 =>  { self.low(val); },
            1 =>  { self.mid(val); },
            2 =>  { self.high(val); },
            _ => { panic!("Parameter doesn't exist.") }
        }
    }

    fn get_param(&mut self, idx: usize) -> SampleType { 
        match idx {
            0 =>  self.low,
            1 =>  self.mid,
            2 =>  self.high,
            _ => { panic!("Parameter doesn't exist.") }
        }
    }
}

impl ToneLMH {
///
///Scale low pass filter output.
///
    pub fn low(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.low = val;
        new
    }

///
///Scale of mid pass notch filter output.
///
    pub fn mid(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.mid = val;
        new
    }

///
///Scale of high pass filter output.
///
    pub fn high(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.high = val;
        new
    }
}
 
 
