use crate::coderef::{CodeRef, EntryRef, ExternRef, GroupRef};
use crate::entries::{CodeGroup, Entry, ExportEntry, ExternEntry};
use crate::value::{closure_prog, Context};
use crate::{BuildError, EvalError, Permutation};
use failure::Error;
use lincoln_common::traits::{Access, StringLike};

#[derive(Serialize, Default)]
pub struct Program {
    pub(crate) entries: Vec<Entry>,
    pub(crate) externs: Vec<ExternEntry>,
    pub(crate) exports: Vec<ExportEntry>,
    pub(crate) groups: Vec<CodeGroup>,
}
impl std::fmt::Debug for Program {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "\nentries:")?;
        for (idx, entry) in self.entries.iter().enumerate() {
            writeln!(fmt, "\t🎯-{}: {}", idx, entry)?;
        }
        writeln!(fmt, "externs:")?;
        for (idx, ext) in self.externs.iter().enumerate() {
            writeln!(fmt, "\t🗨-{}: {}", idx, ext)?;
        }
        writeln!(fmt, "exports:")?;
        for ext in self.exports.iter() {
            writeln!(fmt, "\t🚢-{}", ext)?;
        }
        writeln!(fmt, "groups:")?;
        let grps = self.groups.iter();
        for (idx, grp) in grps.enumerate() {
            write!(fmt, "\t🎎-{}: {{", idx)?;
            let mut grp1 = grp.iter();
            if let Some(ent) = grp1.next() {
                write!(fmt, "{}", ent)?;
            }
            for ent in grp1 {
                write!(fmt, ";{}", ent)?;
            }
            writeln!(fmt, "}}")?;
        }
        Ok(())
    }
}
impl Program {
    pub fn new() -> Self {
        Program {
            entries: vec![],
            externs: vec![],
            exports: vec![],
            groups: vec![],
        }
    }
    pub fn iterate_entries(&self) -> impl Iterator<Item=&Entry> {
        self.entries.iter()
    }
    pub fn iterate_externs(&self) -> impl Iterator<Item=&ExternEntry> {
        self.externs.iter()
    }
    pub fn iterate_groups(&self) -> impl Iterator<Item=&CodeGroup> {
        self.groups.iter()
    }
    pub fn iterate_exports(&self) -> impl Iterator<Item=&ExportEntry> {
        self.exports.iter()
    }
    pub fn add_extern(&mut self, ent: ExternEntry) -> CodeRef {
        let pos = ExternRef::new_coderef(self.externs.len());
        self.externs.push(ent);
        pos
    }
    fn add_entry(&mut self, ent: Entry) -> CodeRef {
        let pos = EntryRef::new_coderef(self.entries.len());
        self.entries.push(ent);
        pos
    }
    pub fn add_return(&mut self, variant: u8) -> CodeRef {
        self.add_entry(Entry::Return { variant })
    }
    pub fn add_jump(&mut self, cont: CodeRef, per: Permutation) -> CodeRef {
        self.add_entry(Entry::Jump { cont, per })
    }
    pub fn add_call(&mut self, call: CodeRef, num_args: u8, cont: GroupRef) -> CodeRef {
        self.add_entry(Entry::Call {
            call,
            cont,
            num_args,
        })
    }
    pub fn add_export(&mut self, name: impl StringLike, g: GroupRef) {
        self.exports.push(ExportEntry {
            name: name.to_string(),
            g,
        })
    }
    pub fn add_empty_group(&mut self) -> GroupRef {
        let pos = GroupRef::new(self.groups.len());
        self.groups.push(smallvec![]);
        pos
    }
    pub fn add_group_entry(&mut self, grp: GroupRef, ent: CodeRef) -> Result<(), BuildError> {
        grp.push_to(ent, self)
    }
    pub fn get_export_ent(
        &self,
        export_label: impl StringLike,
        variant: u8,
    ) -> Result<CodeRef, Error> {
        if let Some(ent) = self
            .exports
            .iter()
            .find(|e| e.name == export_label.as_str())
        {
            ent.g.get_entry(self, variant).map_err(Error::from)
        } else {
            bail!("Export label not found or invalid");
        }
    }
    pub fn get_export(&self, export_label: impl StringLike) -> Result<GroupRef, Error> {
        if let Some(ent) = self
            .exports
            .iter()
            .find(|e| e.name == export_label.as_str())
        {
            Ok(ent.g)
        } else {
            bail!("Export label not found or invalid");
        }
    }
    pub fn run(
        &self,
        ctx: &mut Context,
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
            evalresult = self.eval(ctx, &evalresult)?;
            if let CodeRef::Termination = evalresult {
                break;
            }
            if check_rounds {
                rounds -= 1;
            }
        }
        Ok(())
    }
    pub fn eval(&self, ctx: &mut Context, ent: &CodeRef) -> Result<CodeRef, EvalError> {
        debug!("eval {:?} {}", ent, ctx);
        match ent {
            CodeRef::Entry(ent) => match ent.access(self) {
                Some(Entry::Jump { cont, per }) => {
                    ctx.permutate(*per);
                    Ok(cont.clone())
                }
                Some(Entry::Call {
                    call,
                    cont,
                    num_args,
                }) => {
                    let c2 = ctx.split(*num_args)?;
                    let v = closure_prog(*cont, c2, self)?;
                    ctx.push(v);
                    Ok(call.clone())
                }
                Some(Entry::Return { variant }) => {
                    let v = ctx.pop()?;
                    v.eval(ctx, *variant)
                }
                _ => Err(ent.not_found().into()),
            },
            CodeRef::Extern(ext) => {
                if let Some(ext) = ext.access(self) {
                    match ext {
                        ExternEntry::Eval { ref eval, .. } => eval.eval(ctx),
                        ExternEntry::Value { ref value, .. } => {
                            ctx.expect_args(1)?;
                            let c = ctx.pop()?;
                            ctx.push(value.get_value());
                            c.eval(ctx, 0)
                        }
                    }
                } else {
                    Err(ext.not_found().into())
                }
            }
            CodeRef::Termination => Err(EvalError::EvalOnTermination.into()),
        }
    }
}
