#![deny(bare_trait_objects)]
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
#[macro_use] extern crate smallvec;
#[macro_use] extern crate log;

mod coderef;
mod program;
mod value;
mod permutation;
mod entries;

pub use value::{Context, Value};
pub use coderef::{CodeRef, GroupRef};
pub use entries::ExternEntry;
pub use program::Program;
pub use permutation::{AsPermutation, Permutation};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
