use super::{CodeRef, Context, Value};
use crate::error::EvalError;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::mem::replace;

pub(super) struct WrappedFn<F>(pub(super) String, pub(super) Option<F>);
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
    fn take(&mut self) -> Box<dyn Value> {
        let name = replace(&mut self.0, "Terminate".into());
        Box::new(WrappedFn(name, self.1.take()))
    }
}
