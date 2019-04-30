use crate::coderef::{EntryRef, ExternRef, GroupRef};

use failure::Error;

/// Errors may occur during building
#[derive(Fail, Debug)]
pub enum BuildError {    
    #[fail(display = "Group {:?} not found", _0)]
    GroupNotFound(GroupRef),
    #[fail(display = "Given variant {} exceed limit {}", given, max)]
    VariangOutOfRange { max: u8, given: u8 },
}

/// Errors may occur during evaluation
#[derive(Fail, Debug)]
pub enum EvalError {
    #[fail(display = "Attempt to eval on termination value")]
    EvalOnTermination,
    #[fail(display = "Return to extern value")]
    ReturnToExtern,
    #[fail(display = "Variant out of bound {}/{}", given, max)]
    VariantOutOfBound { given: u8, max: u8 },
    #[fail(display = "Calling a wrapped value")]
    CallingWrapped,
    #[fail(
        display = "Wrong number of arguments, need {} given {}",
        expect, actual
    )]
    UnexpectedArgs { expect: u8, actual: u8 },
    #[fail(display = "{}", _0)]
    CodeRef(CodeRefError),
    #[fail(display = "{}", _0)]
    ValueAccess(ValueAccessError),
    #[fail(display = "{}", _0)]
    External(Error)
}
impl From<CodeRefError> for EvalError {
    fn from(e: CodeRefError) -> Self {
        EvalError::CodeRef(e)
    }
}
impl From<ValueAccessError> for EvalError {
    fn from(e: ValueAccessError) -> Self {
        EvalError::ValueAccess(e)
    }
}

/// Errors may occur when referencing code
#[derive(Fail, Debug)]
pub enum CodeRefError {
    #[fail(display = "Group not found {:?}", index)]
    InvalidGroupIndex { index: GroupRef },
    #[fail(display = "Entry not found: {:?}", index)]
    EntryNotFound { index: EntryRef },
    #[fail(display = "Extern not found: {:?}", index)]
    ExternNotFound { index: ExternRef },
    #[fail(display = "Only extern code reference can be put in auto-wrapping closure")]
    CodeRefNotExtern,
}

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
}
