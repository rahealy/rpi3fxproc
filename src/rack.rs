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

use alloc::rc::Rc;
use alloc::vec::Vec;

use core::cell::RefCell;
use core::default::Default;

use effects;
use effects::SampleType;
use effects::thru::*;
use effects::delay::*;
use effects::pwm::*;
use effects::sine::*;
use effects::constant::*;

use common::buffer::Buffer;
use common::buffer::{Read, Write};

use peripherals::debug;

pub mod connector {
    use effects::SampleType;

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

///
///A connection between two effects in the rack unit is made using a
///connection factory.
///
pub mod connection {
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
            fn to(&'a mut self, _eff: usize, _param: usize) -> 
                ToResult<T>
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
}

///
///Each effect has two lists of connections. From connections provide
///information about which effect is connected to "this" effect and
///which parameter or input it is connected to. The To connections
///provide information about which effects and parameters will receive
///the output from this effect.
///
mod connections {
    use super::connector;
    use alloc::vec::Vec;

    #[derive(Default, Clone)]
    pub struct Effect {
        pub from: Vec<connector::From>, //From connections.
        pub to: Vec<connector::To>,     //To connections.
    }
}

///
///Number of effects contained in a rack unit.
///
const NUM_EFFECTS: usize = 16;

///
///Rack unit master inputs and outputs are fixed.
///
pub const INPUT_A:  usize = 0;
pub const INPUT_B:  usize = 1;
pub const OUTPUT_A: usize = 2;
pub const OUTPUT_B: usize = 3;

///
///Number of parameter values to store + 1 for process value.
///
const NUM_VALUES: usize = 8;

///
///Last value is set aside for passing to the effect's "process" function.
///
const PROCESS_VALUE: usize = NUM_VALUES - 1;

///
///Hardcoded "rack unit" containing effects. 
///
pub struct Unit {
    effects: [Rc<RefCell<dyn effects::Effect>>; NUM_EFFECTS],
    conns:   Vec<connections::Effect>,
    values:  [[SampleType; NUM_VALUES]; NUM_EFFECTS],
    queue:   Vec<usize>,
    ab:      bool,
}

impl Unit {
///
///Process one sample pair's worth through the graph.
///
    pub fn process(&mut self,
                   in_q: &mut Buffer<SampleType>,
                   out_q: &mut Buffer<SampleType>) -> usize
    {
        let mut cnt: usize = 0;

        while !out_q.full_queue() && !in_q.empty_queue() 
        {
            if self.ab { //Both inputs filled. Process.
                self.values[INPUT_B][PROCESS_VALUE] = in_q.dequeue();

                for effect_idx in self.queue.iter() {
                    let mut effect = self.effects[*effect_idx].borrow_mut();

//Update current effect parameters.
                    for conn in self.conns[*effect_idx].from.iter_mut() {
                        if conn.param != PROCESS_VALUE {
                            effect.set_param (
                                conn.param, 
                                self.values[*effect_idx][conn.param]
                            );
//Connections to the same parameter are summed. Clear for next pass.
                            self.values[*effect_idx][conn.param] = SampleType::default();
                        }
                    }

//Process. Clear for next pass.
//                     debug::out("Processing idx: ");
//                     debug::u32hex((*effect_idx) as u32);
//                     debug::out("\r\n");
//                     debug::out("Input = ");
//                     debug::u32hex(self.values[*effect_idx][PROCESS_VALUE] as u32);
//                     debug::out("\r\n");

                    let result = effect.process(self.values[*effect_idx][PROCESS_VALUE]);
                    self.values[*effect_idx][PROCESS_VALUE] = SampleType::default();

//                     debug::out("Output = ");
//                     debug::u32hex(result as u32);
//                     debug::out("\r\n");

//                     debug::out("self.values PROCESS_VALUE = ");
//                     debug::u32hex(self.values[*effect_idx][PROCESS_VALUE] as u32);
//                     debug::out("\r\n");

//Dispatch result to downstream connections.
                    for conn in self.conns[*effect_idx].to.iter_mut() {
//                         debug::out("Conn effect idx: ");
//                         debug::u32hex(conn.effect as u32);
//                         debug::out(" param idx: ");
//                         debug::u32hex(conn.param as u32);
//                         debug::out("\r\n");
//                         debug::out("Value = ");
//                         debug::u32hex(result as u32);
//                         debug::out("\r\n");
//                         debug::out("self.values pre = ");
//                         debug::u32hex(self.values[conn.effect][conn.param] as u32);
//                         debug::out("\r\n");
                        self.values[conn.effect][conn.param] += result;
//                         debug::out("self.values post = ");
//                         debug::u32hex(self.values[conn.effect][conn.param] as u32);
//                         debug::out("\r\n");
                    }
                }

//                 debug::out("[OutputA, OutputB] = [");
//                 debug::u32hex(self.values[OUTPUT_A][PROCESS_VALUE] as u32);
//                 debug::out(",");
//                 debug::u32hex(self.values[OUTPUT_B][PROCESS_VALUE] as u32);
//                 debug::out("]\r\n");
 
                out_q.enqueue(self.values[OUTPUT_A][PROCESS_VALUE]);
                out_q.enqueue(self.values[OUTPUT_B][PROCESS_VALUE]);
                self.values[OUTPUT_A][PROCESS_VALUE] = SampleType::default();
                self.values[OUTPUT_B][PROCESS_VALUE] = SampleType::default();

            } else {
                self.values[INPUT_A][PROCESS_VALUE] = in_q.dequeue();
            }

            cnt += 1;
            self.ab = !self.ab;
        }
        cnt
    }

///
///Unit constructor.
///
    pub fn new() -> Unit {
        let mut unit = Unit {
            effects: [
                Rc::new(RefCell::new(Thru::default())), //INPUT_A
                Rc::new(RefCell::new(Thru::default())), //INPUT_B
                Rc::new(RefCell::new(Thru::default())), //OUTPUT_A
                Rc::new(RefCell::new(Thru::default())), //OUTPUT_B
                Rc::new(RefCell::new(Delay::default())),
                Rc::new(RefCell::new(Delay::default())),
                Rc::new(RefCell::new(Delay::default())), 
                Rc::new(RefCell::new(Delay::default())), 
                Rc::new(RefCell::new(Sine::default())),
                Rc::new(RefCell::new(Sine::default())),
                Rc::new(RefCell::new(Sine::default())),
                Rc::new(RefCell::new(Sine::default())),
                Rc::new(RefCell::new(Pwm::default())),
                Rc::new(RefCell::new(Pwm::default())),
                Rc::new(RefCell::new(Pwm::default())),
                Rc::new(RefCell::new(Pwm::default())),
            ],

            conns:  Vec::<connections::Effect>::default(),
            values: [[SampleType::default(); NUM_VALUES]; NUM_EFFECTS],
            queue:  Vec::<usize>::default(),
            ab: false,
        };

        for i in 0..NUM_EFFECTS {
            unit.effects[i].borrow_mut().reset();
            unit.conns.push(connections::Effect::default());
        }

        unit
    }

    pub fn queue(&mut self, effect: usize) -> Result< (), &'static str > {
        if effect < NUM_EFFECTS {
            if let None = self.queue
                              .iter()
                              .position(|&val| val == effect) 
            {
                self.queue.push(effect);
                Ok(())
            } else {
                Err("rack::unit::queue(): Effect already in queue!")
            }
        } else {
            Err("rack::unit::queue(): Effect not in unit!")
        }
    }
}

impl <'a> connection::factory::From<'a> for Unit {
    fn from(&'a mut self, eff: usize) -> 
        connection::factory::FromResult<Self> 
    {
        if eff < self.effects.len() {
            Ok(( 
                self, 
                connector::From { 
                    effect: eff, 
                    param: PROCESS_VALUE
                } 
            ))
        } else {
            Err("connection::factory::from(): Effect out of range.")
        }
    }
}

impl <'a> connection::factory::To<'a, Unit> for 
    connection::factory::FromResult<'a, Unit>
{
    fn to(&'a mut self, effect: usize, param: usize) -> 
        connection::factory::ToResult<Unit>
    {
        match self {
            Ok((unit, from)) => {
                if effect >= unit.effects.len() {
                    return Err("connection::factory::to(): Effect out of range.")
                }

//Determine if param is an effect parameter or the effect processing input.
                if param < unit.effects[effect]
                               .borrow_mut()
                               .num_params() 
                {
                    Ok ((
                        unit, *from,
                        connector::To { 
                            effect: effect, 
                            param: param
                        }
                    ))
                } else {
                    Ok (( 
                        unit, *from,
                        connector::To { 
                            effect: effect, 
                            param: PROCESS_VALUE
                        }
                    ))
                }
            }
            Err(err) => Err(err)
        }
    }
}

impl <'a> connection::factory::Connect<'a> for
    connection::factory::ToResult<'a, Unit>
{
    fn connect(&'a mut self) -> 
        connection::factory::ConnectResult 
    {
        match self {
            Ok((unit, from, to)) => {
                if let None = unit.conns[from.effect]
                                  .to
                                  .iter()
                                  .position(|&val| val == *to) 
                {
                    if let None = unit.conns[to.effect]
                                      .from
                                      .iter()
                                      .position(|&val| val == *from)
                    {
                        unit.conns[from.effect].to.push(*to);
                        unit.conns[to.effect].from.push (
                            connector::From {
                                effect: from.effect, 
                                param: to.param, //Where to look in the value array for parameter. 
                            }
                        );
                        return Ok(());
                    } else {
                        Err("connection::factory::connect(): To already connected to From!")
                    }
                } else {
                    Err("connection::factory::connect(): From already connected to To!")
                }
            }
            Err(err) => Err(err)
        }
    }
}

impl <'a> connection::factory::Disconnect<'a> for
    connection::factory::ToResult<'a, Unit>
{
    fn disconnect(&'a mut self) -> 
        connection::factory::ConnectResult 
    {
        match self {
            Ok((unit, from, to)) => {
                if let Some(f) = unit.conns[from.effect]
                                     .to
                                     .iter()
                                     .position(|&val| val == *to) 
                {
                    if let Some(t) = unit.conns[to.effect]
                                         .from
                                         .iter()
                                         .position(|&val| val == *from)
                    {
                        unit.conns[from.effect].to.remove(f);
                        unit.conns[to.effect].from.remove(t);
                        return Ok(());
                    } else {
                        Err("connection::factory::disconnect(): To not connected to From!")
                    }
                } else {
                    Err("connection::factory::disconnect(): From not connected to To!")
                }
            }
            Err(err) => Err(err)
        }
    }
}
