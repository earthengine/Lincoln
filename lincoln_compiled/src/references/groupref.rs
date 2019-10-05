use super::CodeRef;
use crate::entries::CodeGroup;
use crate::error::BuildError;
use crate::error::{CodeRefError, EvalError};
use crate::program::Program;

/// A `GroupRef` refers to a group of `CodeRef`, used for
/// `Entry::Call` to implement conditional control flow.
///
#[derive(Copy, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct GroupRef(usize);
impl std::fmt::Debug for GroupRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "G🎎-{}", self.0)
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
            Err(BuildError::GroupNotFound(GroupRef(i)))
        } else {
            let g = &p.groups[i];
            if g.len() <= idx as usize {
                Err(BuildError::VariangOutOfRange {
                    given: idx,
                    max: g.len() as u8,
                })
            } else {
                Ok(g[idx as usize])
            }
        }
    }
    /// Retrive the index value
    pub fn get_index(self) -> usize {
        let GroupRef(i) = self;
        i
    }
    pub(crate) fn push_to(self, c: CodeRef, p: &mut Program) -> Result<(), BuildError> {
        let GroupRef(i) = self;
        if i > p.groups.len() {
            return Err(BuildError::GroupNotFound(GroupRef(i)));
        }
        p.groups[i].push(c);
        Ok(())
    }
    pub(crate) fn get_vec(self, p: &Program) -> Result<CodeGroup, EvalError> {
        let GroupRef(i) = self;
        if i > p.groups.len() {
            Err(EvalError::CodeRef(CodeRefError::InvalidGroupIndex {
                index: self,
            }))
        } else {
            Ok(p.groups[i].clone())
        }
    }
}
