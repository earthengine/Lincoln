#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
#[macro_use] extern crate smallvec;
#[macro_use] extern crate log;

pub mod coderef;
pub mod program;
pub mod value;
pub mod permutation;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
