#![deny(bare_trait_objects)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate smallvec;
#[macro_use]
extern crate log;

mod entries;
mod error;
mod permutation;
mod program;
mod references;
mod value;

pub use entries::{EvalFn, ExternEntry, ValueFn};
pub use error::{BuildError, CodeRefError, EvalError, ValueAccessError};
pub use lincoln_common::traits::Access;
pub use permutation::{AsPermutation, Permutation};
pub use program::Program;
pub use references::{CodeRef, GroupRef};
pub use value::{default_context, native_closure, unwrap, wrap, Context, ContextExt, Value};

/// The crate contains definitions for a "compiled" prgram,
/// which contains low level instructions.

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
