use crate::coderef::ExternRef;
use crate::coderef::{Access, CodeRef, EntryRef, GroupRef};
use crate::permutation::Permutation;
use crate::program_manager::StringLike;
use crate::value::{Context, Value};
use failure::Error;

pub type EvalFn = fn(&'_ Program, Context) -> Result<(CodeRef, Context), Error>;
pub type ValueFn = fn() -> Value;

#[derive(Copy, Clone, Serialize, Debug, Eq, Hash)]
pub enum ExternEntry {
    Eval {
        name: &'static str,
        #[serde(skip_serializing)]
        eval: EvalFn,
    },
    Value {
        name: &'static str,
        #[serde(skip_serializing)]
        value: ValueFn,
    },
}
impl PartialEq for ExternEntry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExternEntry::Eval{ name:n1, eval:e1 }, ExternEntry::Eval{ name:n2, eval:e2 }) => 
                n1==n2 && e1 as *const _ as *const () == e2 as *const _ as *const (),
            (ExternEntry::Value{ name:n1, value:v1 }, ExternEntry::Value{ name:n2, value:v2 })  => 
                n1==n2 && v1 as *const _ as *const () == v2 as *const _ as *const (),
            _ => false
        }
    }
}
impl ExternEntry {
    pub fn name(&self) -> &'static str {
        match self {
            ExternEntry::Eval { name, .. } => name,
            ExternEntry::Value { name, .. } => name,
        }
    }
}
/*pub struct ExternEntry {
    pub name: &'static str,
    #[serde(skip_serializing)]
    pub f: EvalFn,
}*/
#[derive(Clone, Serialize, Debug)]
pub struct GroupEntry {
    pub name: String,
    pub g: GroupRef,
}

#[derive(Serialize, Debug)]
pub struct Program {
    pub entries: Vec<Entry>,
    pub externs: Vec<ExternEntry>,
    pub exports: Vec<GroupEntry>,
    pub groups: Vec<Vec<CodeRef>>,
}

#[derive(Debug, Serialize)]
pub enum Entry {
    Jump {
        cont: CodeRef,
        per: Permutation,
    },
    Call {
        call: CodeRef,
        cont: GroupRef,
        num_args: u8,
    },
    Return {
        variant: u8,
    },
}
impl std::fmt::Display for Entry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Entry::Jump { cont, per } => write!(fmt, "Jump #{} #!{}", cont.get_index(), per),
            Entry::Call {
                call,
                cont,
                num_args,
            } => write!(
                fmt,
                "Call #{} {} #{}",
                call.get_index(),
                num_args,
                cont.get_index()
            ),
            Entry::Return { variant } => write!(fmt, "Return {}", variant),
        }
    }
}

impl Program {
    pub fn new() -> Program {
        Program {
            entries: vec![],
            externs: vec![],
            exports: vec![],
            groups: vec![],
        }
    }
    pub fn add_extern(&mut self, ent: ExternEntry) -> CodeRef {
        let pos = ExternRef::new_coderef(self.externs.len());
        self.externs.push(ent);
        pos
    }
    pub fn add_entry(&mut self, ent: Entry) -> CodeRef {
        let pos = EntryRef::new_coderef(self.entries.len());
        self.entries.push(ent);
        pos
    }
    pub fn add_empty_group(&mut self) -> GroupRef {
        let pos = GroupRef::new(self.groups.len());
        self.groups.push(vec![]);
        pos
    }
    pub fn add_group_entry(&mut self, grp: GroupRef, ent: CodeRef) -> Result<(), Error> {
        grp.push_to(ent, self)
    }
    pub fn run(
        &self,
        ctx: Context,
        export_label: impl StringLike,
        variant: u8,
    ) -> Result<(), Error> {
        if let Some(ent) = self
            .exports
            .iter()
            .find(|e| e.name == export_label.as_ref())
        {
            let ent = ent.g.as_entry_ref(self, variant)?;
            let (mut ent, mut ctx) = self.eval(ctx, &ent)?;
            loop {
                let (ent1, ctx1) = self.eval(ctx, &ent)?;
                if let CodeRef::Termination = ent1 { break }
                ent = ent1;
                ctx = ctx1;
            }
            Ok(())
        } else {
            bail!("invalid export label");
        }
    }
    fn _show_code(&self, code: CodeRef) -> Option<String> {
        match code {
            CodeRef::Entry(ent) => ent.access(self).map(|ent| format!("{}", ent)),
            CodeRef::Extern(ext) => {
                if let Some(ext) = ext.access(self) {
                    if let Some(ext) = self.externs.iter().find(|ext1| *ext1==ext) {
                        return Some(ext.name().into());
                    }
                }
                None
            }
            CodeRef::Termination => Some("Termination".into()),
        }
    }
    pub fn eval(&self, mut ctx: Context, ent: &CodeRef) -> Result<(CodeRef, Context), Error> {
        match ent {
            CodeRef::Entry(ent) => match ent.access(self) {
                Some(Entry::Jump { cont, per }) => {
                    ctx.permutate(*per);
                    Ok((*cont, ctx))
                }
                Some(Entry::Call {
                    call,
                    cont,
                    num_args,
                }) => {
                    let (mut c1, c2) = ctx.split(*num_args);
                    let v = Value::closure(*cont, c2);
                    c1.push(v);
                    Ok((*call, c1))
                }
                Some(Entry::Return { variant }) => {
                    let v = ctx.pop_first()?;
                    v.eval(self, ctx, *variant)
                }
                _ => Err(ent.not_found()),
            },
            CodeRef::Extern(ext) => {
                if let Some(ext) = ext.access(self) {
                    if let ExternEntry::Eval { eval, .. } = ext {
                        eval(self, ctx)
                    } else {
                        bail!("Returning to a value extern")
                    }
                } else {
                    Err(ext.not_found())
                }
            }
            CodeRef::Termination => bail!("Eval on termination"),
        }
    }
}
