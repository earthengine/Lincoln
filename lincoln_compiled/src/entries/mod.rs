use crate::value::{Value, Context};
use crate::error::EvalError;
use crate::permutation::Permutation;
use crate::references::{CodeRef, GroupRef};
use smallvec::SmallVec;

mod wrapped_fn;
mod eval_fn;
mod export_entry;
mod extern_entry;
mod value_fn;

pub use eval_fn::EvalFn;
pub use export_entry::ExportEntry;
pub use extern_entry::ExternEntry;
pub use value_fn::ValueFn;

use wrapped_fn::WrappedFn;

pub(crate) type CodeGroup = SmallVec<[CodeRef; 5]>;

#[derive(Serialize)]
pub enum Entry {
    Jump {
        cont: CodeRef,
        per: Permutation,
    },
    Call {
        call: CodeRef,
        cont: GroupRef,
        num_args: u8,
    },
    Return {
        variant: u8,
    },
}
impl std::fmt::Display for Entry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Entry::Jump { cont, per } => write!(fmt, "Jump {} #!{}({:?})", cont, per, per),
            Entry::Call {
                call,
                cont,
                num_args,
            } => write!(fmt, "Call {} {} {}", call, num_args, cont),
            Entry::Return { variant } => write!(fmt, "Return {}", variant),
        }
    }
}
impl std::fmt::Debug for Entry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}
pub fn native_closure(
    name: impl Into<String>,
    f: impl FnOnce(&mut dyn Context, u8) -> Result<CodeRef, EvalError> + 'static,
) -> Box<dyn Value> {
    Box::new(WrappedFn(name.into(), Some(f)))
}
