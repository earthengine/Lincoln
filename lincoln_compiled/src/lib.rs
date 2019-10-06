#![deny(bare_trait_objects)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate smallvec;
#[macro_use]
extern crate log;

mod closure;
mod entries;
mod error;
mod program;
mod references;

pub use entries::{EvalFn, ExternEntry, ValueFn};
pub use error::{BuildError, CodeRefError, EvalError};
pub use lincoln_common::Access;
pub use program::Program;
pub use references::{CodeRef, GroupRef};
pub use entries::native_closure;
pub use closure::eval_closure;

/// The crate contains definitions for a "compiled" prgram,
/// which contains low level instructions.

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
