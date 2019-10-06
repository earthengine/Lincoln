use crate::{EvalError, ValueAccessError};
use lincoln_common::AnyDebugDisplay;

mod context;
mod traits;
mod wrapped;

pub use traits::{Context, ContextExt, Value};

use context::ContextImpl;
use wrapped::Wrapped;

pub fn wrap<T>(t: T) -> Box<dyn Value>
where
    T: AnyDebugDisplay,
{
    Box::new(Wrapped(Some(t)))
}
pub fn unwrap<T>(v: Box<dyn Value>) -> Result<T, EvalError>
where
    T: AnyDebugDisplay,
{
    v.into_boxed_any()
        .downcast::<Wrapped<T>>()
        .map_err(|_| ValueAccessError::UnwrapNotWrapped("not Wrapped type".into()))?
        .0
        .ok_or_else(|| EvalError::from(ValueAccessError::UnwrapEmptyValue))
}

pub fn default_context() -> Box<dyn Context> {
    Box::new(ContextImpl::default())
}
