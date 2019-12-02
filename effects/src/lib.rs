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
#![no_std]

pub type SampleType = f32;
pub const SAMPLE_RATE: SampleType = 48000.0;
pub const SAMPLE_RATE_USIZE: usize = SAMPLE_RATE as usize;

pub mod thru;
pub mod delay;
pub mod pwm;
pub mod sine;
pub mod constant;

///
///Common trait implemented by all effects.
///
pub trait Effect {
    fn process(&mut self, smpl_in: SampleType) -> SampleType { smpl_in }
    fn reset(&mut self) {}
    fn num_params(&mut self) -> usize { 0 }
    fn set_param(&mut self, _idx: usize, _val: SampleType) {}
    fn get_param(&mut self, _idx: usize) -> SampleType { SampleType::default() }  
}

