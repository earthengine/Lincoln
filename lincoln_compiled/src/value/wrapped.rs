use super::{CodeRef, Context, Value};
use crate::error::EvalError;
use core::fmt::{Debug, Display, Formatter};
use lincoln_common::traits::AnyDebugDisplay;

pub(super) struct Wrapped<T>(pub(super) Option<T>);
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
        match &self.0 {
            Some(v) => write!(fmt, "|{}|", v),
            None => write!(fmt, "|()|"),
        }
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
    fn take(&mut self) -> Box<dyn Value> {
        Box::new(Wrapped(self.0.take()))
    }
}
