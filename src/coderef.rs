use crate::program::ExternEntry;
use crate::program::{Entry, Program};
use failure::Error;

pub trait Access<'a, Source> {
    type Target: 'a;
    fn access<'b>(&self, src: &'b Source) -> Self::Target
    where
        'b: 'a;
}
pub trait AccessMut<'a, Source> {
    type Target: 'a;
    fn access_mut<'b>(&self, src: &'b mut Source) -> Self::Target
    where
        'b: 'a;
}

#[derive(Copy, Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub enum CodeRef {
    Entry(EntryRef),
    Extern(ExternRef),
    Termination,
}
impl CodeRef {
    pub fn get_index(&self) -> usize {
        match self {
            CodeRef::Entry(ent) => ent.0,
            CodeRef::Extern(ExternRef(index)) => *index,
            _ => std::usize::MAX,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct EntryRef(usize);
impl EntryRef {
    pub fn not_found(&self) -> Error {
        format_err!("entry not found: {}", self.0)
    }
    pub fn new_coderef(index: usize) -> CodeRef {
        EntryRef(index).into()
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

#[derive(Copy, Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct ExternRef(usize);
impl ExternRef {
    pub fn not_found(&self) -> Error {
        format_err!("extern reference not found {:?}", self)
    }
    pub fn new_coderef(index: usize) -> CodeRef {
        ExternRef(index).into()
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

#[derive(Copy, Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct GroupRef(usize);
impl GroupRef {
    pub fn new(i: usize) -> GroupRef {
        GroupRef(i)
    }
    pub fn as_entry_ref(self, p: &Program, idx: u8) -> Result<CodeRef, Error> {
        let GroupRef(i) = self;
        let len = p.groups.len();
        if len <= i {
            bail!("Group entry not found {}", i)
        } else {
            let g = &p.groups[i];
            if g.len() <= idx as usize {
                bail!("Variant out of range: given {}, max {}", idx, g.len())
            } else {
                Ok(g[idx as usize])
            }
        }
    }
    pub fn get_index(&self) -> usize {
        let GroupRef(i) = self;
        *i
    }
    pub fn push_to(&self, c: CodeRef, p: &mut Program) -> Result<(),Error> {
        let GroupRef(i) = self;
        Ok(p.groups[*i].push(c))
    }
}

