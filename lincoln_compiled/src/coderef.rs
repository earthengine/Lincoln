use crate::entries::{CodeGroup, Entry, ExternEntry};
use crate::program::Program;
use crate::{BuildError, CodeRefError, EvalError};
use lincoln_common::traits::Access;

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

/// An `EntryRef` refers to an entry of a program.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryRef(pub usize);
impl EntryRef {
    pub fn not_found(&self) -> CodeRefError {
        CodeRefError::EntryNotFound { index: *self }
    }
    pub(crate) fn new_coderef(index: usize) -> CodeRef {
        EntryRef(index).into()
    }
}
impl std::fmt::Debug for EntryRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "🎯-{}", self.0)
    }
}
impl std::fmt::Display for EntryRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}
impl From<EntryRef> for CodeRef {
    fn from(e: EntryRef) -> CodeRef {
        CodeRef::Entry(e)
    }
}
impl<'a> Access<'a, Program> for EntryRef {
    type Target = Option<&'a Entry>;
    fn access<'b>(&self, src: &'b Program) -> Option<&'b Entry>
    where
        'b: 'a,
    {
        let len = src.entries.len();
        if self.0 >= len {
            None
        } else {
            Some(&src.entries[self.0])
        }
    }
}

/// An `ExternRef` refers to an external entry defined in a program.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ExternRef(pub usize);
impl ExternRef {
    pub fn not_found(&self) -> CodeRefError {
        CodeRefError::ExternNotFound { index: *self }
    }
    pub(crate) fn new_coderef(index: usize) -> CodeRef {
        ExternRef(index).into()
    }
}
impl std::fmt::Debug for ExternRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "🗨-{}", self.0)
    }
}
impl std::fmt::Display for ExternRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}
impl From<ExternRef> for CodeRef {
    fn from(e: ExternRef) -> CodeRef {
        CodeRef::Extern(e)
    }
}
impl<'a> Access<'a, Program> for ExternRef {
    type Target = Option<&'a ExternEntry>;
    fn access<'b>(&self, src: &'b Program) -> Option<&'a ExternEntry>
    where
        'b: 'a,
    {
        let len = src.externs.len();
        if self.0 >= len {
            None
        } else {
            Some(&src.externs[self.0])
        }
    }
}

/// A `GroupRef` refers to a group of `CodeRef`, used for
/// `Entry::Call` to implement conditional control flow.
///
#[derive(Copy, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct GroupRef(usize);
impl std::fmt::Debug for GroupRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "🎎-{}", self.0)
    }
}
impl std::fmt::Display for GroupRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}
impl GroupRef {
    pub(crate) fn new(i: usize) -> GroupRef {
        GroupRef(i)
    }
    pub(crate) fn count(self, prog: &Program) -> Option<u8> {
        let GroupRef(i) = self;
        let len = prog.groups.len();
        if len <= i {
            return None;
        }
        Some(prog.groups[i].len() as u8)
    }
    /// From a program, retrive an entry from this group
    ///
    /// p: the program
    /// idx: the index of the group entries
    ///
    pub fn get_entry(self, p: &Program, idx: u8) -> Result<CodeRef, BuildError> {
        let GroupRef(i) = self;
        let len = p.groups.len();
        if len <= i {
            return Err(BuildError::GroupNotFound(GroupRef(i)));
        } else {
            let g = &p.groups[i];
            if g.len() <= idx as usize {
                return Err(BuildError::VariangOutOfRange {
                    given: idx,
                    max: g.len() as u8,
                });
            } else {
                Ok(g[idx as usize].clone())
            }
        }
    }
    /// Retrive the index value
    pub fn get_index(&self) -> usize {
        let GroupRef(i) = self;
        *i
    }
    pub(crate) fn push_to(&self, c: CodeRef, p: &mut Program) -> Result<(), BuildError> {
        let GroupRef(i) = self;
        if *i > p.groups.len() {
            return Err(BuildError::GroupNotFound(GroupRef(*i)));
        }
        Ok(p.groups[*i].push(c))
    }
    pub(crate) fn get_vec(&self, p: &Program) -> Result<CodeGroup, EvalError> {
        let GroupRef(i) = self;
        if *i > p.groups.len() {
            Err(EvalError::CodeRef(CodeRefError::InvalidGroupIndex {
                index: *self,
            }))
        } else {
            Ok(p.groups[*i].clone())
        }
    }
}
