extern crate alloc;

use effects::prelude::*;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use common::buffer::Queue;
use common::buffer::{Read, Write, BUFFER_LEN};
use effect::{SampleType, Effect};
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
    ToneA,
    ToneB,
    DelayA,
    DelayB,
    OutputA,
    OutputB,
    NumEffects,
}

///
///Number of effects contained in a rack unit.
///
const NUM_EFFECTS: usize = EffectIdx::NumEffects as usize;

///
///Number of samples that will be processed each time.
///
pub const PROCESS_BLOCK_LEN: usize = (BUFFER_LEN / 8) * 6;

/**********************************************************************
 * Unit
 **********************************************************************/

///
///Hardcoded "rack unit" containing effects. 
///
pub struct Unit<'a> {
    pub effects: [Rc<RefCell<dyn Effect>>; NUM_EFFECTS],
    pub inputs:  [effect::Inputs; NUM_EFFECTS],
    pub outputs: [Vec<&'a effect::Input>; NUM_EFFECTS],
    pub queue:   Vec<usize>,
}

impl Unit<'_> {
///
///Process one sample pair's worth through the graph.
///
    pub fn process(&self,
                   in_q:  &mut Queue<SampleType>,
                   out_q: &mut Queue<SampleType>)
    {
        use common::buffer::Amount;
//Reset and enqueue input data.
        {
            let mut i_a = self.inputs[EffectIdx::InputA as usize][thru::INPUT].borrow_mut();
            let mut i_b = self.inputs[EffectIdx::InputB as usize][thru::INPUT].borrow_mut();

            i_a.buf.reset();
            i_b.buf.reset();

            for _ in 0..PROCESS_BLOCK_LEN {
                i_a.enqueue(in_q.dequeue());
                i_b.enqueue(in_q.dequeue());
            }
        }

//Process effects queue.
        let mut outputs = effect::Outputs::<'_>::default();

        for e_idx in self.queue.iter() {
//Outputs are references to other effects inputs.
            for i_ref in self.outputs[*e_idx].iter() {
//Borrow input buffer, reset and add to outputs so it can be filled with data. 
                let mut i_queue = i_ref.borrow_mut();
                i_queue.buf.reset();
                outputs.push(i_queue);
            }
//Process.
            self.effects[*e_idx].borrow_mut().process (
                &self.inputs[*e_idx],
                &mut outputs
            );
            outputs.clear();
        }

//Output[A,B] Thru effects are placeholders and not processed
//in the queue. Queue and interleave results.
        {
            let mut o_a = self.inputs[EffectIdx::OutputA as usize][thru::INPUT].borrow_mut();
            let mut o_b = self.inputs[EffectIdx::OutputB as usize][thru::INPUT].borrow_mut();

            for _ in 0..PROCESS_BLOCK_LEN {
                out_q.enqueue(o_a.dequeue());
                out_q.enqueue(o_b.dequeue());
            }
        }
    }


///
///Unit constructor.
///
    pub fn new<'a>() -> Unit<'a> {
        Unit {
            effects: Unit::effects(),
            inputs:  Unit::inputs(),
            outputs: Unit::outputs(),
            queue:   Vec::<usize>::default(),
        }
    }

    fn effects() -> [Rc<RefCell<dyn Effect>>; NUM_EFFECTS] {
        [
            Rc::new(RefCell::new(thru::Processor::default())),    //INPUT_A
            Rc::new(RefCell::new(thru::Processor::default())),    //INPUT_B
//             Rc::new(RefCell::new(thru::Processor::default())),    //INPUT_A
//             Rc::new(RefCell::new(thru::Processor::default())),    //INPUT_B
            Rc::new(RefCell::new(tone::Processor::default())),    //TONE_A
            Rc::new(RefCell::new(tone::Processor::default())),    //TONE_B
            Rc::new(RefCell::new(delay::Processor::default())),   //DELAY_A
            Rc::new(RefCell::new(delay::Processor::default())),   //DELAY_B
            Rc::new(RefCell::new(thru::Processor::default())),    //OUTPUT_A
            Rc::new(RefCell::new(thru::Processor::default())),    //OUTPUT_B
        ]
    }

    fn inputs() -> [effect::Inputs; NUM_EFFECTS] {
        [
            effect::Inputs::default(),
            effect::Inputs::default(),
            effect::Inputs::default(),
            effect::Inputs::default(),
            effect::Inputs::default(),
            effect::Inputs::default(),
            effect::Inputs::default(),
            effect::Inputs::default(),
        ]
    }

    fn outputs<'a>() -> [Vec<&'a effect::Input>; NUM_EFFECTS] {
        [
            Vec::<&effect::Input>::default(),
            Vec::<&effect::Input>::default(),
            Vec::<&effect::Input>::default(),
            Vec::<&effect::Input>::default(),
            Vec::<&effect::Input>::default(),
            Vec::<&effect::Input>::default(),
            Vec::<&effect::Input>::default(),
            Vec::<&effect::Input>::default(),
        ]
    }
}
