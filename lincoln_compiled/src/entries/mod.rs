use crate::coderef::{CodeRef, GroupRef};
use crate::permutation::Permutation;
use smallvec::SmallVec;

mod eval_fn;
mod value_fn;
mod extern_entry;
mod export_entry;

pub use eval_fn::EvalFn;
pub use value_fn::ValueFn;
pub use extern_entry::ExternEntry;
pub use export_entry::ExportEntry;

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
