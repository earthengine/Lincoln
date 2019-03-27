use crate::PreCompileProgram;
use failure::Error;
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
                write!(fmt, "group")?;
                let it = elements.iter().enumerate();
                for (idx, element) in it {
                    write!(fmt, " {}:{}", idx, *element)?;
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
        if let Ok(v) = self.access(pm) {
            v.is_group()
        } else {
            false
        }
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
    type Target = Result<&'a Entry, Error>;
    fn access<'b>(&self, src: &'b PreCompileProgram) -> Self::Target
    where
        'b: 'a,
    {
        src.entry(self.index)
    }
}
impl<'a> AccessMut<'a, PreCompileProgram> for EntryRef {
    type Target = Result<&'a mut Entry, Error>;
    fn access_mut<'b>(&self, src: &'b mut PreCompileProgram) -> Self::Target
    where
        'b: 'a,
    {
        src.entry_mut(self.index)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_debug() {
        use crate::entry::{Entry, EntryRef};
        let ent = Entry::Jmp {
            cont: EntryRef::new(0),
            per: "ba".parse().unwrap(),
        };
        assert_eq!(format!("{:?}", ent), "jmp #0 #!ba");
        let ent = Entry::Call {
            callee: EntryRef::new(1),
            callcnt: 3,
            callcont: EntryRef::new(2),
        };
        assert_eq!(format!("{:?}", ent), "call #1 3 #2");
        let ent = Entry::Ret { variant: 2 };
        assert_eq!(format!("{:?}", ent), "ret 2");
        let ent = Entry::Extern { name: "ext".into() };
        assert_eq!(format!("{:?}", ent), "extern ext");
        let ent = Entry::Group {
            elements: vec![EntryRef::new(3), EntryRef::new(4)],
        };
        assert_eq!(format!("{:?}", ent), "group 0:#3 1:#4");
    }
    #[test]
    fn test_display() {
        use crate::entry::{Entry, EntryRef};
        let ent = Entry::Jmp {
            cont: EntryRef::new(0),
            per: "ba".parse().unwrap(),
        };
        assert_eq!(format!("{}", ent), "jmp #0 #!ba");
        let ent = Entry::Call {
            callee: EntryRef::new(1),
            callcnt: 3,
            callcont: EntryRef::new(2),
        };
        assert_eq!(format!("{}", ent), "call #1 3 #2");
        let ent = Entry::Ret { variant: 2 };
        assert_eq!(format!("{}", ent), "ret 2");
        let ent = Entry::Extern { name: "ext".into() };
        assert_eq!(format!("{}", ent), "extern ext");
        let ent = Entry::Group {
            elements: vec![EntryRef::new(3), EntryRef::new(4)],
        };
        assert_eq!(format!("{}", ent), "group 0:#3 1:#4");
    }
}
