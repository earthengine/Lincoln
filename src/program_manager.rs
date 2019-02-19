use crate::coderef::Access;
use crate::fact_externs::FACT_EXTERNS;
use crate::permutation::Permutation;
use crate::program::Entry as PEntry;
use crate::program::GroupEntry;
use crate::program::Program;
use core::fmt::Display;
use core::fmt::Formatter;
use failure::Error;
use std::collections::HashSet;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

pub trait StringLike: Into<String> + AsRef<str> {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
impl<T> StringLike for T where T: Into<String> + AsRef<str> {}

#[derive(Debug, Serialize, Deserialize)]
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
impl Entry {
    fn is_group(&self) -> bool {
        match self {
            Entry::Group { .. } => true,
            _ => false,
        }
    }
}
impl Display for Entry {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntryRef {
    index: usize,
}
impl EntryRef {
    fn is_group_in(&self, pm: &ProgramManager) -> bool {
        if let Some(v) = self.access(pm) {
            v.is_group()
        } else {
            false
        }
    }
}
impl Display for EntryRef {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "#{}", self.index)
    }
}
impl<'a> Access<'a, ProgramManager> for EntryRef {
    type Target = Option<&'a Entry>;
    fn access<'b>(&self, src: &'b ProgramManager) -> Self::Target
    where
        'b: 'a,
    {
        if self.index < src.entries.len() {
            Some(&src.entries[self.index])
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramManager {
    //prog: Program,
    defined_ent: HashMap<String, EntryRef>,
    entries: Vec<Entry>,
    exports: HashSet<String>,
}
impl Display for ProgramManager {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(fmt, "ProgramManager {{")?;
        writeln!(fmt, "\tdefined_ent: {{")?;
        for ent in self.defined_ent.iter() {
            writeln!(fmt, "\t\t{}: {}", ent.0, ent.1)?;
        }
        writeln!(fmt, "\t}}")?;
        writeln!(fmt, "\tentries: {{")?;
        for entry in self.entries.iter().enumerate() {
            writeln!(fmt, "\t\t{}: {}", entry.0, entry.1)?;
        }
        writeln!(fmt, "\t}}")?;
        writeln!(fmt, "}}")
    }
}

impl ProgramManager {
    pub fn new() -> ProgramManager {
        ProgramManager {
            //prog: Program::new(),
            defined_ent: HashMap::new(),
            entries: vec![],
            exports: HashSet::new(),
        }
    }
    pub fn set_export(&mut self, label: impl StringLike) -> Result<(), Error> {
        if self.defined_ent.contains_key(label.as_ref()) {
            self.exports.insert(label.into());
            Ok(())
        } else {
            bail!("label not found");
        }
    }
    fn define_ent_internal(
        &mut self,
        label: impl StringLike,
        ent: Entry,
    ) -> Result<EntryRef, Error> {
        let idx = self.entries.len();
        let ret = EntryRef { index: idx };
        if let Some(ext) = self.defined_ent.get(label.as_ref()) {
            let ent_orig = &self.entries[ext.index];
            if let Entry::Extern { .. } = ent_orig {
                self.entries[ext.index] = ent;
            } else {
                bail!("Redefine entry that is not extern: {} => {}", ent_orig, ent);
            }
        } else {
            self.entries.push(ent);
            self.defined_ent.insert(label.into(), ret);
        }
        Ok(ret)
    }

    fn define_extern_or_entry(&mut self, name: impl StringLike) -> Result<EntryRef, Error> {
        if let Some(ent) = self.defined_ent.get(name.as_ref()) {
            return Ok(*ent);
        }
        let ent = Entry::Extern {
            name: name.to_string(),
        };
        let r = self.define_ent_internal(name.to_string(), ent)?;
        Ok(r)
    }
    fn define_jmp_internal(
        &mut self,
        name: impl StringLike,
        cont: EntryRef,
        per: Permutation,
    ) -> Result<EntryRef, Error> {
        let ent = Entry::Jmp { cont, per: per };
        self.define_ent_internal(name.into(), ent)
    }
    fn define_call_internal(
        &mut self,
        name: impl StringLike,
        callee: EntryRef,
        callcnt: u8,
        callcont: EntryRef,
    ) -> Result<EntryRef, Error> {
        let ent = Entry::Call {
            callee,
            callcnt,
            callcont,
        };
        self.define_ent_internal(name.into(), ent)
    }
    fn define_ret_internal(
        &mut self,
        name: impl StringLike,
        variant: u8,
    ) -> Result<EntryRef, Error> {
        let ent = Entry::Ret { variant };
        self.define_ent_internal(name.into(), ent)
    }
    fn define_group_internal(
        &mut self,
        name: impl StringLike,
        elements: Vec<EntryRef>,
    ) -> Result<EntryRef, Error> {
        let ent = Entry::Group { elements };
        self.define_ent_internal(name.into(), ent)
    }

    pub fn define_jmp(
        &mut self,
        label: impl StringLike,
        cont: impl StringLike,
        per: impl StringLike,
    ) -> Result<EntryRef, Error> {
        let cont = self.define_extern_or_entry(cont)?;
        self.define_jmp_internal(label, cont, Permutation::from_str(per.as_ref())?)
    }
    pub fn define_call(
        &mut self,
        label: impl StringLike,
        callee: impl StringLike,
        callcnt: u8,
        callcont: impl StringLike,
    ) -> Result<EntryRef, Error> {
        let callee = self.define_extern_or_entry(callee)?;
        let callcont = self.define_extern_or_entry(callcont)?;
        self.define_call_internal(label, callee, callcnt, callcont)
    }
    pub fn define_ret(&mut self, label: impl StringLike, variant: u8) -> Result<EntryRef, Error> {
        self.define_ret_internal(label, variant)
    }
    pub fn define_group(
        &mut self,
        label: impl StringLike,
        elements: &[impl StringLike],
    ) -> Result<EntryRef, Error> {
        let elements = collect_successful(
            elements
                .iter()
                .map(|e| self.define_extern_or_entry(e.to_string())),
        )?;
        self.define_group_internal(label, elements)
    }
    pub fn externs(&self) -> Vec<(String, EntryRef)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, e)| match e {
                Entry::Extern { name, .. } => Some((name.clone(), EntryRef { index })),
                _ => None,
            })
            .collect()
    }
    pub fn find_refs(&self, seed: &HashSet<EntryRef>) -> HashSet<EntryRef> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, e)| match e {
                Entry::Call {
                    callee, callcont, ..
                } => {
                    if seed.iter().find(|e| *e == callee).is_some()
                        && (callcont.is_group_in(self)
                            || seed.iter().find(|e| *e == callcont).is_some())
                    {
                        Some(EntryRef { index })
                    } else {
                        None
                    }
                }
                Entry::Jmp { cont, .. } => {
                    if let Some(_) = seed.iter().find(|e| *e == cont) {
                        Some(EntryRef { index })
                    } else {
                        None
                    }
                }
                Entry::Group { elements } => {
                    for element in elements {
                        if let None = seed.iter().find(|e| *e == element) {
                            return None;
                        }
                    }
                    Some(EntryRef { index })
                }
                Entry::Ret { .. } => Some(EntryRef { index }),
                _ => None,
            })
            .collect()
    }
    fn dependency_sort(&self) -> BTreeMap<usize, HashSet<EntryRef>> {
        let mut r = BTreeMap::new();
        let mut s1: HashSet<EntryRef> = self.externs().iter().map(|(_, e)| *e).collect();

        r.insert(0, s1.clone());
        let mut i = 1;
        loop {
            let s2 = self.find_refs(&s1);
            let s2: HashSet<EntryRef> = s2.difference(&s1).map(|e| *e).collect();
            if s2.len() == 0 {
                break;
            }
            s1 = s1.union(&s2).map(|e| *e).collect();
            r.insert(i, s2);
            i = i + 1;
        }
        r
    }
    pub fn compile(&self) -> Result<Program, Error> {
        let mut prog = Program::new();
        let mut coderef_map = HashMap::new();
        let mut groupdef_map = HashMap::new();
        let ds = self.dependency_sort();
        for (level, entries) in ds {
            for entry in entries {
                let entryref = entry;
                let entry = entry
                    .access(&self)
                    .ok_or(format_err!("Invalid entry ref for PM"))?;
                match entry {
                    Entry::Extern { name } => {
                        if let Some(e) = FACT_EXTERNS.iter().find(|e| (*e).name() == name) {
                            let e = (*e).clone();
                            coderef_map.insert(entryref, prog.add_extern(e));
                        } else {
                            bail!("Extern entry not found");
                        }
                    }
                    Entry::Ret { variant } => {
                        coderef_map.insert(
                            entryref,
                            prog.add_entry(PEntry::Return { variant: *variant }),
                        );
                    }
                    Entry::Jmp { cont, per } => {
                        let cont = *coderef_map.get(cont).ok_or(format_err!(
                            "Dependency error: cont for Jmp is undefined: {}, {}",
                            cont,
                            level
                        ))?;
                        coderef_map
                            .insert(entryref, prog.add_entry(PEntry::Jump { cont, per: *per }));
                    }
                    Entry::Call {
                        callee,
                        callcnt,
                        callcont,
                    } => {
                        let call = *coderef_map.get(callee).ok_or(format_err!(
                            "Dependency error: callee for Call is undefined"
                        ))?;
                        let cont = groupdef_map.get(callcont);
                        match cont {
                            Some(cont) => {
                                coderef_map.insert(
                                    entryref,
                                    prog.add_entry(PEntry::Call {
                                        call,
                                        num_args: *callcnt,
                                        cont: *cont,
                                    }),
                                );
                            }
                            None => {
                                let grp = prog.add_empty_group();
                                groupdef_map.insert(*callcont, grp);
                                if let Some(cont) = coderef_map.get(callcont) {
                                    prog.add_group_entry(grp, *cont)?;
                                }
                                coderef_map.insert(
                                    entryref,
                                    prog.add_entry(PEntry::Call {
                                        call,
                                        num_args: *callcnt,
                                        cont: grp,
                                    }),
                                );
                            }
                        }
                    }
                    Entry::Group { elements } => {
                        let grp = if let Some(grp) = groupdef_map.get(&entryref) {
                            *grp
                        } else {
                            let grp = prog.add_empty_group();
                            groupdef_map.insert(entryref, grp);
                            grp
                        };
                        for element in elements {
                            let element = coderef_map.get(element).ok_or(format_err!(
                                "Deoendency error: group element is not defined"
                            ))?;
                            prog.add_group_entry(grp, *element)?;
                        }
                        groupdef_map.insert(entryref, grp);
                    }
                }
            }
        }
        for export in self.exports.iter() {
            let name = export.clone();
            let ent = self.defined_ent.get(export).expect("Invalid export");
            match ent.access(self) {
                Some(Entry::Group { .. }) => {
                    let grp = groupdef_map.get(ent).expect("group not found");
                    prog.exports.push(GroupEntry { name, g: *grp })
                }
                _ => {
                    let grp = prog.add_empty_group();
                    let ent = coderef_map.get(ent).expect("entry not found");
                    prog.add_group_entry(grp, *ent)?;
                    prog.exports.push(GroupEntry { name, g: grp })
                }
            }
        }
        Ok(prog)
    }
}

fn collect_successful<T, E>(i: impl Iterator<Item = Result<T, E>>) -> Result<Vec<T>, E> {
    let mut d = vec![];
    for r in i {
        d.push(r?);
    }
    Ok(d)
}
