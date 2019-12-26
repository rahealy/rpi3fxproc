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

extern crate alloc;

use alloc::vec::Vec;
use common::prelude::*;

///
///All effects use this as a sample type.
///
pub type SampleType = f64;

///
///All effects expect this sample rate in hz.
///
pub const SAMPLE_RATE: SampleType = 48000.0;

///
///Sample rate defined as a usize for convenience.
///
pub const SAMPLE_RATE_USIZE: usize = SAMPLE_RATE as usize;

///
///Maximum number of inputs and of outputs (to) is restricted.
///
const MAX_PARAMS: usize = 8;

/**********************************************************************
 * Inputs / Outputs
 *********************************************************************/ 

///
///Sample buffer.
///
type Queue = common::buffer::Queue<SampleType>;

use alloc::rc::Rc;
use core::cell::RefCell;
use core::cell::RefMut;
use core::ops::Deref;
use core::ops::DerefMut;


/**********************************************************************
 * Input(s)
 *********************************************************************/

pub type Input = Rc<RefCell<Queue>>;

///
///Inputs for processors.
///
pub struct Inputs ([Input; MAX_PARAMS]);

impl Deref for Inputs {
    type Target = [Input; MAX_PARAMS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Inputs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Inputs {
    fn default() -> Self {
        Inputs (
            [
                Rc::new(RefCell::new(Queue::default())),
                Rc::new(RefCell::new(Queue::default())),
                Rc::new(RefCell::new(Queue::default())),
                Rc::new(RefCell::new(Queue::default())),
                Rc::new(RefCell::new(Queue::default())),
                Rc::new(RefCell::new(Queue::default())),
                Rc::new(RefCell::new(Queue::default())),
                Rc::new(RefCell::new(Queue::default())),
            ]
        )
    }
}


/**********************************************************************
 * Outputs
 *********************************************************************/

///
///Outputs for processor.
///
//#[derive(Default)]
pub struct Outputs<'a> (Vec<RefMut<'a, Queue>>);

impl <'a> Deref for Outputs<'a> {
    type Target = Vec<RefMut<'a, Queue>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl <'a> DerefMut for Outputs<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl <'a> Default for Outputs<'a> {
    fn default() -> Self {
        let mut v = Outputs(Vec::<RefMut<'a, Queue>>::default());
        v.reserve(MAX_PARAMS);
        v
    }
}

/**********************************************************************
 * Effect
 *********************************************************************/

///
///Common trait implemented by all effects.
///
pub trait Effect {
    fn process(&mut self, inputs: &Inputs, outputs: &mut Outputs) {
        let i = inputs[0].borrow(); //One input.

        for i_idx in i.iter_idx() {
            let smpl = i.buf.get(i_idx);
            for o in outputs.iter_mut() { //Many outputs.
                o.enqueue(smpl);
            }
        }
    }

    fn reset(&mut self) {}
    fn num_params(&self) -> usize { 1 }
    fn set_param(&mut self, _idx: usize, _val: SampleType) {}
    fn get_param(&mut self, _idx: usize) -> SampleType { SampleType::default() }  
}


/**********************************************************************
 * Connector
 *********************************************************************/

pub mod connector {
    ///
    ///Connector from a specific effect.
    ///
    #[derive(Default, Clone, Copy, PartialEq)]
    pub struct From {
        pub effect: usize, //Index into rack effects array.
        pub param: usize,  //Set to the "To" effect parameter.
    }

    ///
    ///Connector to a specific effect and its input.
    ///
    #[derive(Default, Clone, Copy, PartialEq)]
    pub struct To {
        pub effect: usize, //Index into rack effects array.
        pub param: usize,  //Parameter offset.
    } 
}

pub mod connection {
    use super::Vec;
    use super::connector;

    ///
    ///Connection factory makes/breaks connections using dotted convention:
    ///
    /// if let Err(err) = unit.from(1)
    ///                       .to(2, 2)
    ///                       .connect();
    /// { ... }
    ///
    pub mod factory {
        use super::connector;

        //From
        pub type FromResult<'a, T> = 
            Result< (&'a mut T, connector::From), &'static str >;

        pub trait From<'a>
        {
            fn from(&'a mut self, _effect: usize) -> 
                FromResult<Self> 
            {
                Err("connection::factory::from() not implemented.")
            }
        }

        //To
        pub type ToResult<'a, T> = 
            Result< (&'a mut T, connector::From, connector::To), &'static str >;

        pub trait To<'a, T>
        {
            fn to(&'a mut self, _effect: usize, _param: usize) -> 
                ToResult<'_, T>
            {
                Err("connection::factory::to() not implemented.")
            }
        }

        //Connect
        pub type ConnectResult = Result< (), &'static str >;

        pub trait Connect<'a>
        {
            fn connect(&'a mut self) -> ConnectResult {
                Err("connection::factory::connect() not implemented.")
            }
        }

    //Disconnect
        pub trait Disconnect<'a>
        {
            fn disconnect(&'a mut self) -> ConnectResult {
                Err("connection::factory::disconnect() not implemented.")
            }
        }
    }

/*********************************************************************
 * List
 ********************************************************************/
    ///
    ///List of connections from one effect to another effect.
    ///
    pub struct List {
        pub from: Vec<connector::From>, //From connections.
        pub to: Vec<connector::To>,     //To connections.
    }

    impl Default for List {
        fn default() -> Self {
            List {
                from: Vec::<connector::From>::default(),
                to: Vec::<connector::To>::default()
            }
        }
    }
}





#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
