use crate::entries::{CodeGroup, Entry, ExternEntry};
use crate::program::Program;
use failure::Error;
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
        match self {
            CodeRef::Entry(e) => write!(fmt, "^{:?}", e),
            CodeRef::Extern(e) => write!(fmt, "^{:?}", e),
            CodeRef::Termination => write!(fmt, "^⟂"),
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
    pub(crate) fn get_index(&self) -> usize {
        match self {
            CodeRef::Entry(ent) => ent.0,
            CodeRef::Extern(ExternRef(index)) => *index,
            _ => std::usize::MAX,
        }
    }
}

/// An `EntryRef` refers to an entry of a program.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryRef(pub usize);
impl EntryRef {
    pub fn not_found(&self) -> Error {
        format_err!("entry not found: {}", self.0)
    }
    pub fn new_coderef(index: usize) -> CodeRef {
        EntryRef(index).into()
    }
}
impl std::fmt::Debug for EntryRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "#{}", self.0)
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
    pub fn not_found(&self) -> Error {
        format_err!("extern reference not found {:?}", self)
    }
    pub fn new_coderef(index: usize) -> CodeRef {
        ExternRef(index).into()
    }
}
impl std::fmt::Debug for ExternRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "@{}", self.0)
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
        write!(fmt, "%{}", self.0)
    }
}
impl GroupRef {
    /// Create a new group reference
    ///
    pub fn new(i: usize) -> GroupRef {
        GroupRef(i)
    }

    /// From a program, retrive an entry from this group
    ///
    /// p: the program
    /// idx: the index of the group entries
    ///
    pub fn get_entry(self, p: &Program, idx: u8) -> Result<CodeRef, Error> {
        let GroupRef(i) = self;
        let len = p.groups.len();
        if len <= i {
            bail!("Group entry not found {}", i)
        } else {
            let g = &p.groups[i];
            if g.len() <= idx as usize {
                bail!("Variant out of range: given {}, max {}", idx, g.len())
            } else {
                Ok(g[idx as usize].clone())
            }
        }
    }
    pub fn get_index(&self) -> usize {
        let GroupRef(i) = self;
        *i
    }
    pub(crate) fn push_to(&self, c: CodeRef, p: &mut Program) -> Result<(), Error> {
        let GroupRef(i) = self;
        if *i > p.groups.len() {
            bail!("Invalid group index {}", i)
        }
        Ok(p.groups[*i].push(c))
    }
    pub(crate) fn get_vec(&self, p: &Program) -> Result<CodeGroup, Error> {
        let GroupRef(i) = self;
        if *i > p.groups.len() {
            bail!("Invalid group index {}", i)
        }
        Ok(p.groups[*i].clone())
    }
}
