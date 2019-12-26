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


/***********************************************************************
 * Offset
 **********************************************************************/

#[derive(Default,Copy,Clone)]
pub struct Offset (pub usize);

impl From<usize> for Offset {
    fn from(val: usize) -> Self {
        Offset(val)
    }
}

impl Offset {
    #[inline]
    pub fn inc(&mut self, lim: usize) -> &mut Self {
        let mut new = self;
        new.0 += 1;
        if new.0 == lim {
            new.0 = 0; 
        }
        new
    }

    pub fn add(&mut self, val: usize) -> &mut Self {
        let mut new = self;
        new.0 += val;
        new
    }

    ///
    ///If offset is >= limit then wrap.
    ///
    pub fn wrap(&mut self, lim: usize) -> &mut Self {
        let mut new = self;
        let m = (new.0 + 1) % lim;
        if m > 0 {
            new.0 = m - 1;
        }
        new
    }

    #[inline]
    pub fn val(&self) -> usize {
        self.0
    }
}
