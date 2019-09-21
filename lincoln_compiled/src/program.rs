use crate::coderef::{CodeRef, EntryRef, ExternRef, GroupRef};
use crate::entries::{CodeGroup, Entry, ExportEntry, ExternEntry};
use crate::value::{closure_prog, Context, ContextExt};
use crate::{BuildError, EvalError, Permutation};
use failure::Error;
use lincoln_common::traits::{Access, StringLike};

/// A compiled lincoln program
///
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
    /// Create a new empty program
    pub fn new() -> Self {
        Program {
            entries: vec![],
            externs: vec![],
            exports: vec![],
            groups: vec![],
        }
    }
    /// Iterate all entries
    pub fn iterate_entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }
    /// Iterate all external entries
    pub fn iterate_externs(&self) -> impl Iterator<Item = &ExternEntry> {
        self.externs.iter()
    }
    /// Iterate all groups
    pub fn iterate_groups(&self) -> impl Iterator<Item = &CodeGroup> {
        self.groups.iter()
    }
    /// Iterate all exports
    pub fn iterate_exports(&self) -> impl Iterator<Item = &ExportEntry> {
        self.exports.iter()
    }
    /// Add a new external entry
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
    /// Add a return instruction
    ///
    /// variant: the variant to call on return
    ///
    pub fn add_return(&mut self, variant: u8) -> CodeRef {
        self.add_entry(Entry::Return { variant })
    }
    /// Add a jump instruction
    /// The instruction of the jump target will receive the same
    /// number of arguments as the jumping instruction.
    ///
    /// cont: the next instruction to jump to
    /// per: the permutation to be performed before the jump
    ///
    pub fn add_jump(&mut self, cont: CodeRef, per: Permutation) -> CodeRef {
        self.add_entry(Entry::Jump { cont, per })
    }
    /// Add a call instruction
    /// Note: the instruction/entry to call must accept exactly `num_args + 1` variables
    ///
    /// call: the instruction to call
    /// num_args: the number of values in the context to keep
    /// cont: the instruction group to receive the result
    ///
    pub fn add_call(&mut self, call: CodeRef, num_args: u8, cont: GroupRef) -> CodeRef {
        self.add_entry(Entry::Call {
            call,
            cont,
            num_args,
        })
    }
    /// Set a entry group to be exported as a name
    ///
    /// name: the name of the exported entry
    /// g: the entry group to be exported
    ///
    pub fn add_export(&mut self, name: impl StringLike, g: GroupRef) {
        self.exports.push(ExportEntry {
            name: name.to_string(),
            g,
        })
    }
    /// Create a new empty group and return its reference.
    ///
    pub fn add_empty_group(&mut self) -> GroupRef {
        let pos = GroupRef::new(self.groups.len());
        self.groups.push(smallvec![]);
        pos
    }
    /// Add a new entry to an existing group
    ///
    /// grp: refers to the existing group
    /// ent: the new entry
    ///
    pub fn add_group_entry(&mut self, grp: GroupRef, ent: CodeRef) -> Result<(), BuildError> {
        grp.push_to(ent, self)
    }
    /// Find an export by name, and receive an entry from its variants
    ///
    /// export_label: the name of the export
    /// variant: the variant to return
    ///
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
    /// Find an export but only returns an entry group
    ///
    /// export_label: the name of the export
    ///
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
    /// Run the program
    ///
    /// ctx: the values given to run
    /// export_label: the name of the exported entry to be run into
    /// variant: the variant of the exported entry to run
    /// rounds: None if run to the end is required, otherwise the maximum steps to tun
    ///
    pub fn run(
        &self,
        ctx: &mut dyn Context,
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
    /// Evaluate the program for one step only
    ///
    /// ctx: the values given to evaluate
    /// ent: the current code entry
    ///
    /// returns: the next code entry, or an error
    pub fn eval(&self, ctx: &mut dyn Context, ent: &CodeRef) -> Result<CodeRef, EvalError> {
        debug!("eval {:?} {}", ent, ctx);
        match ent {
            CodeRef::Entry(ent) => match ent.access(self) {
                Some(Entry::Jump { cont, per }) => {
                    ctx.permutate(*per);
                    Ok(*cont)
                }
                Some(Entry::Call {
                    call,
                    cont,
                    num_args,
                }) => {
                    let c2 = ctx.split(*num_args)?;
                    let v = closure_prog(*cont, c2, self)?;
                    ctx.push(v);
                    Ok(*call)
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
            CodeRef::Termination => Err(EvalError::EvalOnTermination),
        }
    }
}
