use crate::connector;

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
    use super::Effect;

//From
    pub type FromResult<'a, T> = 
        Result< (&'a mut T, connector::From), &'static str >;

    pub trait From<'a>
    {
        fn from(&'a mut self, _effect: Effect) -> 
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
        fn to(&'a mut self, _effect: Effect, _param: usize) -> 
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
