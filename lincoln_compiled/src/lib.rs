#![deny(bare_trait_objects)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate smallvec;
#[macro_use]
extern crate log;

mod coderef;
mod entries;
mod permutation;
mod program;
mod value;

pub use coderef::{CodeRef, GroupRef};
pub use entries::{EvalFn, ValueFn};
pub use entries::ExternEntry;
pub use lincoln_common::traits::Access;
pub use permutation::{AsPermutation, Permutation};
pub use program::Program;
pub use value::{Context, Value, wrap, unwrap, native_closure};

/// The crate contains definitions for a "compiled" prgram,
/// which contains low level instructions.

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
