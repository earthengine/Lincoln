use super::{EvalFn, ValueFn};
use core::hash::{Hash, Hasher};

/// An `ExternEntry` refer to a function provided by the external function.
///
#[derive(Serialize)]
pub enum ExternEntry {
    Eval {
        name: String,
        #[serde(skip_serializing)]
        eval: EvalFn,
    },
    Value {
        name: String,
        #[serde(skip_serializing)]
        value: ValueFn,
    },
}
impl std::fmt::Debug for ExternEntry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}
impl std::fmt::Display for ExternEntry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExternEntry::Eval { name, .. } => write!(fmt, "🌐-{}", name),
            ExternEntry::Value { name, .. } => write!(fmt, "🌏-{}", name),
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
    pub fn name(&self) -> &str {
        match self {
            ExternEntry::Eval { name, .. } => name,
            ExternEntry::Value { name, .. } => name,
        }
    }
}
