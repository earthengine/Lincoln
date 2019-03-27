use crate::PreCompileProgram;
use lincoln_common::traits::{Access, AccessMut};
use lincoln_compiled::Permutation;
use std::fmt::{Debug, Display, Formatter};

/// 5 types of entries correspond to 5 different
/// instructions the use type.
///
/// Jmp: permutate the context to have specific order,
///      then jump to another entry (not a group!)
///
/// Call: keep a specific amount of values from context, then
///       build a closure with the rest, and add the closure into
///       the context. Finally, jump to another entry.
///
/// Ret: The first variable must be a closure to extract the context.
///      The specified variant of the closure group entry
///      will be invoked.
///
/// Group: Define a group of entries. Being used to create closures.
///
/// Extern: Denotes a function that to be executed in the outside world.
///
#[derive(Serialize, Deserialize)]
pub enum Entry {
    Jmp {
        cont: EntryRef,
        per: Permutation,
    },
    Call {
        callee: EntryRef,
        callcnt: u8,
        callcont: EntryRef,
    },
    Ret {
        variant: u8,
    },
    Group {
        elements: Vec<EntryRef>,
    },
    Extern {
        name: String,
    },
}
impl Debug for Entry {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}
impl Entry {
    fn is_group(&self) -> bool {
        match self {
            Entry::Group { .. } => true,
            _ => false,
        }
    }
}
impl Display for Entry {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self {
            Entry::Jmp { cont, per } => write!(fmt, "jmp {} #!{}", cont, per),
            Entry::Call {
                callee,
                callcnt,
                callcont,
            } => write!(fmt, "call {} {} {}", callee, callcnt, callcont),
            Entry::Ret { variant } => write!(fmt, "ret {}", variant),
            Entry::Group { elements } => {
                write!(fmt, "group ")?;
                for (idx, element) in elements.iter().enumerate() {
                    write!(fmt, "{}:{} ", idx, *element)?;
                }
                Ok(())
            }
            Entry::Extern { name } => write!(fmt, "extern {}", name),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct EntryRef {
    index: usize,
}
impl EntryRef {
    pub fn is_group_in(&self, pm: &PreCompileProgram) -> bool {
        if let Some(v) = self.access(pm) {
            v.is_group()
        } else {
            false
        }
    }
    pub fn is_in(&self, pm: &PreCompileProgram) -> bool {
        self.index < pm.entries.len()
    }
    pub fn new(index: usize) -> Self {
        EntryRef { index }
    }
}
impl Debug for EntryRef {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}
impl Display for EntryRef {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "#{}", self.index)
    }
}
impl<'a> Access<'a, PreCompileProgram> for EntryRef {
    type Target = Option<&'a Entry>;
    fn access<'b>(&self, src: &'b PreCompileProgram) -> Self::Target
    where
        'b: 'a,
    {
        if self.is_in(src) {
            Some(&src.entries[self.index])
        } else {
            None
        }
    }
}
impl<'a> AccessMut<'a, PreCompileProgram> for EntryRef {
    type Target = Option<&'a mut Entry>;
    fn access_mut<'b>(&self, src: &'b mut PreCompileProgram) -> Self::Target
    where
        'b: 'a,
    {
        if self.is_in(src) {
            Some(&mut src.entries[self.index])
        } else {
            None
        }
    }
}
