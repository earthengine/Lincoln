use super::CodeRef;
use crate::entries::ExternEntry;
use crate::error::CodeRefError;
use crate::program::Program;
use lincoln_common::Access;

/// An `ExternRef` refers to an external entry defined in a program.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ExternRef(pub usize);
impl ExternRef {
    pub fn not_found(self) -> CodeRefError {
        CodeRefError::ExternNotFound { index: self }
    }
    pub(crate) fn new_coderef(index: usize) -> CodeRef {
        ExternRef(index).into()
    }
}
impl std::fmt::Debug for ExternRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "X🗨-{}", self.0)
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
