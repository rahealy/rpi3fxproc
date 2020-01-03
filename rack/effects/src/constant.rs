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

use super::SampleType;
use crate::Effect;
use common::prelude::*;

/***********************************************************************
 * Constant
 **********************************************************************/

#[derive(Default)]
pub struct Constant {
    pub val: SampleType, //Feedback.
}

impl Effect for Constant {
///
///Process.
///
    fn process(&mut self,
               inputs: &[Buffer<SampleType>],
               outputs: &mut [Buffer<SampleType>])
    {
        while !outputs[0].full_queue() {
            outputs[0].enqueue(self.val);
        }
    }

///
///Reset delay to defaults.
///
    fn reset(&mut self) {
        self.val = SampleType::default();
    }

    fn num_params(&self) -> usize { 1 }

    fn set_param(&mut self, _idx: usize, val: SampleType) {
        self.val = val;
    }

    fn get_param(&mut self, _idx: usize) -> SampleType { 
        self.val
    }
}

impl Constant {
///
///Set constant value.
///
    pub fn val(&mut self, val: SampleType) -> &mut Self {
        let mut new = self;
        new.val = val;
        new
    }
}
 