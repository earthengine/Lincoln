use crate::coderef::{CodeRef, GroupRef};
use crate::entries::ExternEntry;
use crate::program::Program;
use crate::{EvalError, ValueAccessError};
use lincoln_common::traits::{Access, AnyDebugDisplay};

mod traits;
mod closure;
mod context;
mod wrapped;
mod wrapped_fn;

pub use traits::{Value, Context, ContextExt};


use closure::Closure;
use wrapped::Wrapped;
use wrapped_fn::WrappedFn;
use context::ContextImpl;

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

    Ok(Box::new(Closure(ent.get_vec(prog)?, ctx)))
}

pub fn wrap<T>(t: T) -> Box<dyn Value>
where
    T: AnyDebugDisplay,
{
    Box::new(Wrapped(t))
}
pub fn unwrap<T>(v: Box<dyn Value>) -> Result<T, EvalError>
where
    T: AnyDebugDisplay,
{
    let fail = EvalError::from(ValueAccessError::UnwrapNotWrapped(
        "fail into_wrapped".into(),
    ));
    Ok(v.into_wrapped()
        .ok_or(fail)?
        .into_boxed_any()
        .downcast::<Wrapped<T>>()
        .map_err(|_| ValueAccessError::UnwrapNotWrapped("not Wrapped type".into()))?
        .0)
}

pub fn native_closure(
    name: impl Into<String>,
    f: impl FnOnce(&mut dyn Context, u8) -> Result<CodeRef, EvalError> + 'static,
) -> Box<dyn Value> {
    Box::new(WrappedFn(name.into(), f))
}

pub fn default_context() -> Box<dyn Context>{
    Box::new(ContextImpl::default())
}
