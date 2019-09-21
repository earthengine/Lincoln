use core::fmt::Display;
use core::fmt::Formatter;
use crate::error::EvalError;
use crate::coderef::CodeRef;
use super::{Value, Context};
use core::fmt::Debug;

pub(super) struct WrappedFn<F>(pub(super) String, pub(super) F);
impl<F> Debug for WrappedFn<F>
where
    F: FnOnce(&mut dyn Context, u8) -> Result<CodeRef, EvalError>,
{
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}
impl<F> Display for WrappedFn<F>
where
    F: FnOnce(&mut dyn Context, u8) -> Result<CodeRef, EvalError>,
{
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}
impl<F> Value for WrappedFn<F>
where
    F: FnOnce(&mut dyn Context, u8) -> Result<CodeRef, EvalError> + 'static,
{
    fn eval(self: Box<Self>, ctx: &mut dyn Context, variant: u8) -> Result<CodeRef, EvalError> {
        self.1(ctx, variant)
    }
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>> {
        None
    }
}
