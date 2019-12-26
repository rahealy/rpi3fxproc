#![no_std]

pub mod unit;
pub use effect;
pub use effects;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
