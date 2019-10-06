#![deny(bare_trait_objects)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
#[macro_use] extern crate smallvec;

mod traits;
mod value;
mod permutation;
pub use traits::{Access, AccessMut, AnyDebugDisplay, StringLike};
pub use permutation::{AsPermutation, Permutation};
pub use value::{Value, ContextExt, Context, wrap, unwrap, default_context };

/// Errors may occurs when working with values
#[derive(Fail, Debug)]
pub enum ValueAccessError {
    #[fail(display = "Splitting context at {}, total {}", at, total)]
    SplitOutOfRange { at: u8, total: u8 },

    #[fail(display = "Pop from empty context")]
    PopFromEmpty,

    #[fail(display = "Attempt to unwrap a value that was not wrapped - {}", _0)]
    UnwrapNotWrapped(String),

    #[fail(display = "Unwrapping closure with non-empty context")]
    UnwrappingNonEmptyClosure,

    #[fail(display = "Unwrapping closure with multiple variants")]
    UnwrappingMultivariantClosure,

    #[fail(display = "Only value externs can be put in auto-wrapping closure")]
    ExternNotValue,

    #[fail(display = "Cannot turn into wrapped")]
    CannotTurnIntoWrapped,

    #[fail(display = "Unwrap empty value")]
    UnwrapEmptyValue,

    #[fail(
        display = "Wrong number of arguments, need {} given {}",
        expect, actual
    )]
    UnexpectedArgs { expect: u8, actual: u8 },
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
