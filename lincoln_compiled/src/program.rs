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
impl Program {
    pub fn new() -> Self {
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
    pub fn add_export(&mut self, name: impl Into<String>, g: GroupRef) {
        self.exports.push(ExportEntry {
            name: name.into(),
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
            .find(|e| e.name == export_label.as_ref())
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
    pub fn eval(&self, mut ctx: Context, ent: &CodeRef) -> Result<(CodeRef, Context), EvalError> {
        debug!("eval {:?} {:?}", ent, ctx);
        match ent {
            CodeRef::Entry(ent) => match ent.access(self) {
                Some(Entry::Jump { cont, per }) => {
                    ctx.permutate(*per);
                    Ok((cont.clone(), ctx))
                }
                Some(Entry::Call {
                    call,
                    cont,
                    num_args,
                }) => {
                    let (mut c1, c2) = ctx.split(*num_args)?;
                    let v = closure_prog(*cont, c2, self)?;
                    c1.push(v);
                    Ok((call.clone(), c1))
                }
                Some(Entry::Return { variant }) => {
                    let v = ctx.pop()?;
                    v.eval(self, ctx, *variant)
                }
                _ => Err(ent.not_found().into()),
            },
            CodeRef::Extern(ext) => {
                if let Some(ext) = ext.access(self) {
                    if let ExternEntry::Eval { ref eval, .. } = ext {
                        eval.eval(self, ctx)
                    } else {
                        Err(EvalError::ReturnToExtern.into())
                    }
                } else {
                    Err(ext.not_found().into())
                }
            }
            CodeRef::Termination => Err(EvalError::EvalOnTermination.into()),
        }
    }
}
