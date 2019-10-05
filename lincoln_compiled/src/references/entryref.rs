use super::CodeRef;
use crate::entries::Entry;
use crate::error::CodeRefError;
use crate::program::Program;
use lincoln_common::traits::Access;

/// An `EntryRef` refers to an entry of a program.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryRef(pub usize);
impl EntryRef {
    pub fn not_found(self) -> CodeRefError {
        CodeRefError::EntryNotFound { index: self }
    }
    pub(crate) fn new_coderef(index: usize) -> CodeRef {
        EntryRef(index).into()
    }
}
impl std::fmt::Debug for EntryRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "E🎯-{}", self.0)
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
