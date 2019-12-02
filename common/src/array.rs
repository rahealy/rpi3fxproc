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

///
///Fixed length array.
///

use core::slice::IterMut;
use core::iter::Take;

const SIZE: usize = 512;

#[derive(Copy, Clone)]
pub struct Array<S> where 
    S:Default + Copy + Clone + PartialEq
{
    cnt: usize,
    arr: [S; SIZE],
}

impl <I> Default for Array::<I> where
    I:Default + Copy + Clone + PartialEq
{
    fn default() -> Array<I> {
        Array::<I> {
            cnt: usize::default(),
            arr: [I::default(); SIZE]
        }
    }
}

impl <I> Array<I> where 
    I:Default + Copy + Clone + PartialEq
{
    pub fn has(&self, conn: &I) -> Result<usize, ()> {
        for i in 0..self.cnt {
            if self.arr[i] == *conn {
                return Ok(i);
            }
        }
        return Err(());
    }
    
    pub fn add(&mut self, conn: &I) {
        if self.cnt < SIZE {
            if let Err(_) = self.has(conn) {
                self.arr[self.cnt] = *conn;
                self.cnt += 1;
            }
        }
    }

    pub fn rmv_idx(&mut self, idx: usize) {
        if self.cnt > 0 {
            if idx < self.cnt {
                for j in idx..self.cnt - 1 {
                    self.arr[j] = self.arr[j + 1];
                }
                self.cnt -= 1;
            }
        }
    }

    pub fn rmv(&mut self, conn: &I) {
        if self.cnt > 0 {
            if let Ok(i) = self.has(conn) {
                for j in i..self.cnt - 1 {
                    self.arr[j] = self.arr[j + 1];
                }
                self.cnt -= 1;
            }
        }
    }
    
    pub fn iter_mut(&mut self) -> Take<IterMut<'_, I>> {
        self.arr.iter_mut().take(self.cnt)
    }
}
 
