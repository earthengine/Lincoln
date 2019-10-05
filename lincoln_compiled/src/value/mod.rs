use crate::entries::ExternEntry;
use crate::program::Program;
use crate::references::{CodeRef, GroupRef};
use crate::{EvalError, ValueAccessError};
use lincoln_common::traits::{Access, AnyDebugDisplay};

mod closure;
mod context;
mod traits;
mod wrapped;
mod wrapped_fn;

pub use traits::{Context, ContextExt, Value};

use closure::Closure;
use context::ContextImpl;
use wrapped::Wrapped;
use wrapped_fn::WrappedFn;

/// Build a closure value from a group reference, a context and program
///
pub(crate) fn closure_prog(
    ent: GroupRef,
    ctx: Box<dyn Context>,
    prog: &Program,
) -> Result<Box<dyn Value>, EvalError> {
    if let Some(1) = ent.count(prog) {
        if let Ok(CodeRef::Extern(ext)) = ent.get_entry(prog, 0) {
            if let Some(ExternEntry::Value { value, .. }) = ext.access(prog) {
                ctx.expect_args(0)?;
                return Ok(value.get_value());
            }
        }
    }

    Ok(Box::new(Closure {
        tags: ent.get_vec(prog)?.into_vec(),
        context: ctx,
    }))
}

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
    let fail = EvalError::from(ValueAccessError::UnwrapNotWrapped(
        "fail into_wrapped".into(),
    ));
    v.into_wrapped()
        .ok_or(fail)?
        .into_boxed_any()
        .downcast::<Wrapped<T>>()
        .map_err(|_| ValueAccessError::UnwrapNotWrapped("not Wrapped type".into()))?
        .0
        .ok_or_else(|| EvalError::from(ValueAccessError::UnwrapEmptyValue))
}

pub fn native_closure(
    name: impl Into<String>,
    f: impl FnOnce(&mut dyn Context, u8) -> Result<CodeRef, EvalError> + 'static,
) -> Box<dyn Value> {
    Box::new(WrappedFn(name.into(), Some(f)))
}

pub fn default_context() -> Box<dyn Context> {
    Box::new(ContextImpl::default())
}
