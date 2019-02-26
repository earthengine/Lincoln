use crate::coderef::{CodeRef, GroupRef};
use crate::program::Program;
use crate::value::{Value, Context};
use crate::permutation::Permutation;
use smallvec::SmallVec;
use failure::Error;

use std::hash::{Hash, Hasher};

pub type CodeGroup = SmallVec<[CodeRef; 5]>;

pub type EvalFn = fn(&'_ Program, Context) -> Result<(CodeRef, Context), Error>;
pub type ValueFn = fn() -> Value;

#[derive(Copy, Clone, Serialize)]
pub enum ExternEntry {
    Eval {
        name: &'static str,
        #[serde(skip_serializing)]
        eval: EvalFn,
    },
    Value {
        name: &'static str,
        #[serde(skip_serializing)]
        value: ValueFn,
    },
}
impl std::fmt::Debug for ExternEntry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ExternEntry::Eval { name, .. } => write!(fmt, "@{}", name),
            ExternEntry::Value { name, .. } => write!(fmt, "@{}", name),
        }
    }
}
impl Hash for ExternEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ExternEntry::Eval { name, eval } => (name, eval as *const _ as *const ()).hash(state),
            ExternEntry::Value { name, value } => {
                (name, value as *const _ as *const ()).hash(state)
            }
        }
    }
}
impl Eq for ExternEntry {}
impl PartialEq for ExternEntry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ExternEntry::Eval { name: n1, eval: e1 },
                ExternEntry::Eval { name: n2, eval: e2 },
            ) => n1 == n2 && e1 as *const _ as *const () == e2 as *const _ as *const (),
            (
                ExternEntry::Value {
                    name: n1,
                    value: v1,
                },
                ExternEntry::Value {
                    name: n2,
                    value: v2,
                },
            ) => n1 == n2 && v1 as *const _ as *const () == v2 as *const _ as *const (),
            _ => false,
        }
    }
}
impl ExternEntry {
    pub fn name(&self) -> &'static str {
        match self {
            ExternEntry::Eval { name, .. } => name,
            ExternEntry::Value { name, .. } => name,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct ExportEntry {
    pub name: String,
    pub g: GroupRef,
}
impl std::fmt::Debug for ExportEntry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}: {:?}", self.name, self.g)
    }
}

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
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Entry::Jump { cont, per } => write!(fmt, "Jump #{} #!{}", cont.get_index(), per),
            Entry::Call {
                call,
                cont,
                num_args,
            } => write!(
                fmt,
                "Call #{} {} #{}",
                call.get_index(),
                num_args,
                cont.get_index()
            ),
            Entry::Return { variant } => write!(fmt, "Return {}", variant),
        }
    }
}
impl std::fmt::Debug for Entry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Entry::Jump { cont, per } => write!(fmt, "Jump {:?} !{:?}({})", cont, per, per),
            Entry::Call {
                call,
                cont,
                num_args,
            } => write!(fmt, "Call {:?} {:?} {:?}", call, num_args, cont),
            Entry::Return { variant } => write!(fmt, "Return {}", variant),
        }
    }
}