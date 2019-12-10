
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
