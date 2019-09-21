use crate::error::EvalError;
use crate::coderef::CodeRef;
use lincoln_common::traits::AnyDebugDisplay;
use super::{Value, Context};
use core::fmt::{Display, Formatter, Debug};

pub(super) struct Wrapped<T>(pub(super) T);
impl<T> Debug for Wrapped<T>
where
    T: Debug,
{
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "|{:?}|", self.0)
    }
}
impl<T> Display for Wrapped<T>
where
    T: Display,
{
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "|{}|", self.0)
    }
}
impl<T> Value for Wrapped<T>
where
    T: AnyDebugDisplay,
{
    fn eval(self: Box<Self>, _: &mut dyn Context, _: u8) -> Result<CodeRef, EvalError> {
        Err(EvalError::CallingWrapped)
    }
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>> {
        Some(self)
    }
}
