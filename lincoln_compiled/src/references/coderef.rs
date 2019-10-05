use super::entryref::EntryRef;
use super::ExternRef;

/// CodeRef is a type refer to a single executable entry.
/// This can be either a entry of a program, an external
/// entry point defined within/without the program,
/// or indicate the end of execution.
///
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum CodeRef {
    /// An entry refers to a entry point of a program.
    Entry(EntryRef),
    /// Refers to an external function entry defined in a program.
    Extern(ExternRef),
    /// Indicate the end of execution.
    Termination,
}
impl std::hash::Hash for CodeRef {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        use CodeRef::*;
        match self {
            Entry(e) => e.hash(state),
            Extern(e) => e.hash(state),
            _ => "".hash(state),
        }
    }
}
impl Eq for CodeRef {}
impl PartialEq for CodeRef {
    fn eq(&self, other: &Self) -> bool {
        use CodeRef::*;
        match (self, other) {
            (Entry(e1), Entry(e2)) => e1 == e2,
            (Extern(e1), Extern(e2)) => e1 == e2,
            (Termination, Termination) => true,
            _ => false,
        }
    }
}
impl std::fmt::Debug for CodeRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}
impl std::fmt::Display for CodeRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CodeRef::Entry(e) => write!(fmt, "{}", e),
            CodeRef::Extern(e) => write!(fmt, "{}", e),
            CodeRef::Termination => write!(fmt, "🛑"),
        }
    }
}
impl CodeRef {
    pub fn entry(index: usize) -> Self {
        CodeRef::Entry(EntryRef(index))
    }
    pub fn ext(index: usize) -> Self {
        CodeRef::Extern(ExternRef(index))
    }
}
