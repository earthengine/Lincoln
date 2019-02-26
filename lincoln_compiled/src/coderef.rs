use crate::program::{CodeGroup, Entry, EvalFn, ExternEntry, Program};
use crate::value::{Context, Value};
use lincoln_common::traits::Access;
use failure::Error;

#[derive(Copy, Clone, Serialize)]
pub enum CodeRef {
    Entry(EntryRef),
    Extern(ExternRef),
    ExternFn(&'static str, #[serde(skip_serializing)] EvalFn),
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
            ExternFn(n, f) => {
                n.hash(state);
                (f as *const _ as *const ()).hash(state)
            }
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
            (ExternFn(n1, f1), ExternFn(n2, f2)) => {
                n1 == n2 && f1 as *const _ as *const () == f2 as *const _ as *const ()
            }
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
            CodeRef::ExternFn(name, _) => write!(fmt, "^{}", name),
            CodeRef::Termination => write!(fmt, "^⟂"),
        }
    }
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

#[derive(Copy, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryRef(usize);
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

#[derive(Copy, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct ExternRef(usize);
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

#[derive(Copy, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct GroupRef(usize);
impl std::fmt::Debug for GroupRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "%{}", self.0)
    }
}
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
    pub fn push_to(&self, c: CodeRef, p: &mut Program) -> Result<(), Error> {
        let GroupRef(i) = self;
        if *i > p.groups.len() {
            bail!("Invalid group index {}", i)
        }
        Ok(p.groups[*i].push(c))
    }
    pub fn get_vec(&self, p: &Program) -> Result<CodeGroup, Error> {
        let GroupRef(i) = self;
        if *i > p.groups.len() {
            bail!("Invalid group index {}", i)
        }
        Ok(p.groups[*i].clone())
    }
    pub fn create_closure(&self, p: &Program, ctx: Context) -> Result<Value, Error> {
        let GroupRef(i) = self;
        if *i > p.groups.len() {
            bail!("Invalid group index {}", i)
        }
        Ok(Value::closure(&p.groups[*i], ctx))
    }
}
