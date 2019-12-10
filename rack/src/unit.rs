extern crate alloc;

use effects::prelude::*;
use effects::SampleType;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use common::buffer::Buffer;
use common::buffer::{Read, Write};
use peripherals::debug;

/**********************************************************************
 * Effect
 *********************************************************************/

///
///Rack unit master inputs and outputs are fixed.
///
#[derive(Copy, Clone, PartialEq)]
pub enum EffectIdx {
    InputA = 0,
    InputB,
    DelayA,
    DelayB,
    ToneA,
    ToneB,
    OutputA,
    OutputB,
    NumEffects,
}

///
///Number of effects contained in a rack unit.
///
const NUM_EFFECTS: usize = EffectIdx::NumEffects as usize;

///
///Number of parameter values to store + 1 for process value.
///
const NUM_VALUES: usize = 8;

///
///Last value is set aside for passing to the effect's "process" function.
///
const PROCESS_VALUE: usize = NUM_VALUES - 1;


/**********************************************************************
 * Connector
 **********************************************************************/

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


/**********************************************************************
 * Connection
 **********************************************************************/

///
///A connection between two effects in the rack unit is made using a
///connection factory.
///
pub mod connection {
    use super::connector;
    use super::EffectIdx;

///
///Represents a connection
///
    #[derive(Default, Clone)]
    pub struct Connection {
        pub from: connector::From, //From connections.
        pub to: connector::To,     //To connections.
    }

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
        use super::EffectIdx;
        use super::Connection;

//From
        pub type FromResult<'a, T> = 
            Result< (&'a mut T, connector::From), &'static str >;

        pub trait From<'a>
        {
            fn from(&'a mut self, _effect: EffectIdx) -> 
                FromResult<Self> 
            {
                Err("connection::factory::from() not implemented.")
            }
        }

//To
        pub type ToResult<'a, T> = 
            Result< (&'a mut T, Connection), &'static str >;

        pub trait To<'a, T>
        {
            fn to(&'a mut self, _effect: EffectIdx, _param: usize) -> 
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


struct Connections {
    pub from: Vec<connector::From>, //From connections.
    pub to: Vec<connector::To>,     //To connections.
}

impl Default for Connections {
    fn default() -> Self {
        Connections {
            from: Vec::<connector::From>::default(),
            to: Vec::<connector::To>::default()
        }
    }
}


/**********************************************************************
 * Unit
 **********************************************************************/
 

///
///Hardcoded "rack unit" containing effects. 
///
pub struct Unit {
    effects: [Rc<RefCell<dyn effects::Effect>>; NUM_EFFECTS],
    conns:   [Connections; NUM_EFFECTS],
    values:  [[SampleType; NUM_VALUES]; NUM_EFFECTS],
    queue:   Vec<EffectIdx>,
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
                self.values[EffectIdx::InputB as usize][PROCESS_VALUE] = in_q.dequeue();

                for e in self.queue.iter() {
                    let mut effect = self.effects[*e as usize].borrow_mut();

//Update current effect parameters.
                    for conn in self.conns[*e as usize].from.iter_mut() {
//                         debug::out("Update effect: ");
//                         debug::u32hex((*e as usize) as u32);
//                         debug::out("\r\n");
// 
//                         debug::out("Using parameter: ");
//                         debug::u32hex(conn.param as u32);
//                         debug::out("\r\n");

                        if conn.param != PROCESS_VALUE {
                            effect.set_param (
                                conn.param, 
                                self.values[*e as usize][conn.param]
                            );
//Connections to the same parameter are summed. Clear for next pass.
                            self.values[*e as usize][conn.param] = SampleType::default();
                        }
                    }

//Process. Clear for next pass.
//                     debug::out("Processing effect: ");
//                     debug::u32hex((*e as usize) as u32);
//                     debug::out("\r\n");
//                     debug::out("Input = ");
//                     debug::u32hex(self.values[*e as usize][PROCESS_VALUE] as u32);
//                     debug::out("\r\n");

                    let result = effect.process(self.values[*e as usize][PROCESS_VALUE]);
                    self.values[*e as usize][PROCESS_VALUE] = SampleType::default();

//                     debug::out("Output = ");
//                     debug::u32hex(result as u32);
//                     debug::out("\r\n");

//                     debug::out("self.values PROCESS_VALUE = ");
//                     debug::u32hex(self.values[*e as usize][PROCESS_VALUE] as u32);
//                     debug::out("\r\n");

//Dispatch result to downstream connections.
                    for conn in self.conns[*e as usize].to.iter_mut() {
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

                out_q.enqueue(self.values[EffectIdx::OutputA as usize][PROCESS_VALUE]);
                out_q.enqueue(self.values[EffectIdx::OutputB as usize][PROCESS_VALUE]);
                self.values[EffectIdx::OutputA as usize][PROCESS_VALUE] = SampleType::default();
                self.values[EffectIdx::OutputB as usize][PROCESS_VALUE] = SampleType::default();

            } else {
                self.values[EffectIdx::InputA as usize][PROCESS_VALUE] = in_q.dequeue();
            }

            cnt += 1;
            self.ab = !self.ab;
        }
        cnt
    }

    fn effects() -> [Rc<RefCell<dyn effects::Effect>>; NUM_EFFECTS] {
        [
            Rc::new(RefCell::new(Thru::default())),    //INPUT_A
            Rc::new(RefCell::new(Thru::default())),    //INPUT_B
            Rc::new(RefCell::new(Delay::default())),   //DELAY_A
            Rc::new(RefCell::new(Delay::default())),   //DELAY_B
            Rc::new(RefCell::new(ToneLMH::default())), //TONE_A
            Rc::new(RefCell::new(ToneLMH::default())), //TONE_B
            Rc::new(RefCell::new(Thru::default())),    //OUTPUT_A
            Rc::new(RefCell::new(Thru::default())),    //OUTPUT_B
        ]
    }
    
    fn conns() -> [Connections; NUM_EFFECTS] {
        [
            Connections::default(),
            Connections::default(),
            Connections::default(),
            Connections::default(),
            Connections::default(),
            Connections::default(),
            Connections::default(),
            Connections::default(),
        ]
    }

///
///Unit constructor.
///
    pub fn new() -> Unit {
        let unit = Unit {
            effects: Unit::effects(),
            conns:  Unit::conns(),
            values: [[SampleType::default(); NUM_VALUES]; NUM_EFFECTS],
            queue:  Vec::<EffectIdx>::default(),
            ab: false,
        };

        for i in 0..NUM_EFFECTS {
            unit.effects[i].borrow_mut().reset();
        }

        unit
    }

    pub fn queue(&mut self, effect: EffectIdx) -> Result< (), &'static str > {
        if let None = self.queue
                        .iter()
                        .position(|&val| val == effect) 
        {
            self.queue.push(effect);
            Ok(())
        } else {
            Err("rack::unit::queue(): Effect already in queue!")
        }
    }
}

impl <'a> connection::factory::From<'a> for Unit {
    fn from(&'a mut self, effect: EffectIdx) -> 
        connection::factory::FromResult<Self> 
    {
        Ok(( 
            self, 
            connector::From { 
                effect: effect as usize, 
                param: PROCESS_VALUE
            } 
        ))
    }
}

impl <'a> connection::factory::To<'a, Unit> for 
    connection::factory::FromResult<'a, Unit>
{
    fn to(&'a mut self, effect: EffectIdx, param: usize) -> 
        connection::factory::ToResult<Unit>
    {
        match self {
            Ok((unit, from)) => {
//Determine if param is an effect parameter or the effect processing input.
                let to = if param < unit.effects[effect as usize]
                                        .borrow_mut()
                                        .num_params()
                {                    
                    connector::To { 
                        effect: effect as usize, 
                        param: param,
                    }
                } else {
                    connector::To { 
                        effect: effect as usize, 
                        param: PROCESS_VALUE,
                    }
                };

                Ok ((
                    unit, 
                    connection::Connection {
                        from: *from,
                        to: to,
                    }
                ))
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
            Ok((unit, conn)) => {
                if let None = unit.conns[conn.from.effect]
                                  .to
                                  .iter()
                                  .position(|&val| val == conn.to) 
                {
                    if let None = unit.conns[conn.to.effect]
                                      .from
                                      .iter()
                                      .position(|&val| val == conn.from)
                    {
                        unit.conns[conn.from.effect].to.push(conn.to);
                        unit.conns[conn.to.effect].from.push (
                            connector::From {
                                effect: conn.from.effect, 
                                param: conn.to.param, //Where to look in the value array for parameter. 
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
            Ok((unit, conn)) => {
                if let Some(f) = unit.conns[conn.from.effect]
                                     .to
                                     .iter()
                                     .position(|&val| val == conn.to) 
                {
                    if let Some(t) = unit.conns[conn.to.effect]
                                         .from
                                         .iter()
                                         .position(|&val| val == conn.from)
                    {
                        unit.conns[conn.from.effect].to.remove(f);
                        unit.conns[conn.to.effect].from.remove(t);
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

