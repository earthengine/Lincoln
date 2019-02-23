use crate::compiled::coderef::ExternRef;
use crate::compiled::coderef::{CodeRef, EntryRef, GroupRef};
use crate::compiled::value::{Context, Value};
use crate::permutation::Permutation;
use crate::traits::{Access, StringLike};
use core::hash::Hash;
use core::hash::Hasher;
use failure::Error;
use smallvec::SmallVec;

pub type CodeGroup = SmallVec<[CodeRef; 5]>;

pub type EvalFn = fn(&'_ Program, Context) -> Result<(CodeRef, Context), Error>;
pub type ValueFn = fn() -> Value;

#[derive(Copy, Clone, Serialize)]
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
impl std::fmt::Debug for ExternEntry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ExternEntry::Eval { name, .. } => write!(fmt, "@{}", name),
            ExternEntry::Value { name, .. } => write!(fmt, "@{}", name),
        }
    }
}
impl Hash for ExternEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ExternEntry::Eval { name, eval } => (name, eval as *const _ as *const ()).hash(state),
            ExternEntry::Value { name, value } => {
                (name, value as *const _ as *const ()).hash(state)
            }
        }
    }
}
impl Eq for ExternEntry {}
impl PartialEq for ExternEntry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ExternEntry::Eval { name: n1, eval: e1 },
                ExternEntry::Eval { name: n2, eval: e2 },
            ) => n1 == n2 && e1 as *const _ as *const () == e2 as *const _ as *const (),
            (
                ExternEntry::Value {
                    name: n1,
                    value: v1,
                },
                ExternEntry::Value {
                    name: n2,
                    value: v2,
                },
            ) => n1 == n2 && v1 as *const _ as *const () == v2 as *const _ as *const (),
            _ => false,
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

#[derive(Clone, Serialize)]
pub struct ExportEntry {
    pub name: String,
    pub g: GroupRef,
}
impl std::fmt::Debug for ExportEntry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}: {:?}", self.name, self.g)
    }
}

#[derive(Serialize, Default)]
pub struct Program {
    pub entries: Vec<Entry>,
    pub externs: Vec<ExternEntry>,
    pub exports: Vec<ExportEntry>,
    pub groups: Vec<CodeGroup>,
}
impl std::fmt::Debug for Program {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "\nentries:")?;
        for (idx, entry) in self.entries.iter().enumerate() {
            writeln!(fmt, "\t#{}: {:?}", idx, entry)?;
        }
        writeln!(fmt, "externs:")?;
        for (idx, ext) in self.externs.iter().enumerate() {
            writeln!(fmt, "\t@{}: {:?}", idx, ext)?;
        }
        writeln!(fmt, "exports:")?;
        for ext in self.exports.iter() {
            writeln!(fmt, "\t{:?}", ext)?;
        }
        writeln!(fmt, "groups:")?;
        let grps = self.groups.iter();
        for (idx, grp) in grps.enumerate() {
            write!(fmt, "\t%{}: [", idx)?;
            let mut grp1 = grp.iter();
            if let Some(ent) = grp1.next() {
                write!(fmt, "{:?}", ent)?;
            }
            for ent in grp1 {
                write!(fmt, ", {:?}", ent)?;
            }
            writeln!(fmt, "]")?;
        }
        writeln!(fmt, "")
    }
}

#[derive(Serialize)]
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
impl std::fmt::Debug for Entry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Entry::Jump { cont, per } => write!(fmt, "Jump {:?} !{:?}({})", cont, per, per),
            Entry::Call {
                call,
                cont,
                num_args,
            } => write!(fmt, "Call {:?} {:?} {:?}", call, num_args, cont),
            Entry::Return { variant } => write!(fmt, "Return {}", variant),
        }
    }
}

impl Program {
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
        self.groups.push(smallvec![]);
        pos
    }
    pub fn add_group_entry(&mut self, grp: GroupRef, ent: CodeRef) -> Result<(), Error> {
        grp.push_to(ent, self)
    }
    pub fn get_export_ent(&self, export_label: impl StringLike, variant: u8) -> Result<CodeRef, Error> {
        if let Some(ent) = self
            .exports
            .iter()
            .find(|e| e.name == export_label.as_ref())
        {
            ent.g.as_entry_ref(self, variant)
        } else {
            bail!("Export label not found or invalid");
        }
    }
    pub fn get_export(&self, export_label: impl StringLike) -> Result<GroupRef, Error> {
        if let Some(ent) = self
            .exports
            .iter()
            .find(|e| e.name == export_label.as_ref())
        {
            Ok(ent.g)
        } else {
            bail!("Export label not found or invalid");
        }
    }
    pub fn run(
        &self,
        ctx: Context,
        export_label: impl StringLike,
        variant: u8,
        rounds: Option<usize>,
    ) -> Result<(), Error> {
        let ent = self.get_export_ent(export_label, variant)?;
        let mut evalresult = self.eval(ctx, &ent)?;
        let (check_rounds, mut rounds) = (rounds.is_some(), rounds.unwrap_or(0));
        loop {
            if check_rounds && rounds == 0 {
                break;
            };
            if check_rounds {
                print!("{}: ", rounds);
            }
            evalresult = self.eval(evalresult.1, &evalresult.0)?;
            if let CodeRef::Termination = evalresult.0 {
                break;
            }
            if check_rounds {
                rounds -= 1;
            }
        }
        Ok(())
    }
    pub fn eval(&self, mut ctx: Context, ent: &CodeRef) -> Result<(CodeRef, Context), Error> {
        debug!("eval {:?} {:?}", ent, ctx);
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
                    let v = Value::closure_prog(*cont, c2, self)?;
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
            CodeRef::ExternFn(_, f) => f(self, ctx),
            CodeRef::Termination => bail!("Eval on termination"),
        }
    }
}
