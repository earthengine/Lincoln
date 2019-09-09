use crate::codemap::CodeMap;
use crate::entry::{Entry, EntryRef};
use core::fmt::{Debug, Display, Formatter};
use failure::Error;
use lincoln_common::traits::{Access, AccessMut, StringLike};
use lincoln_compiled::{AsPermutation, ExternEntry, Permutation, Program};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Serialize, Deserialize, Default)]
pub struct PreCompileProgram {
    defined_ent: BTreeMap<String, EntryRef>,
    entries: Vec<Entry>,
    exports: BTreeSet<String>,
}
impl Display for PreCompileProgram {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        for (idx, ent) in self.entries.iter().enumerate() {
            let label = self.find_name(EntryRef::new(idx))?;
            match ent {
                Entry::Jmp { cont, per } => {
                    write!(fmt, "{}: jmp {} #!{}", label, self.find_name(*cont)?, per)?
                }
                Entry::Call {
                    callee,
                    callcnt,
                    callcont,
                } => writeln!(
                    fmt,
                    "{}: call {} {} {}",
                    label,
                    self.find_name(*callee)?,
                    callcnt,
                    self.find_name(*callcont)?
                )?,
                Entry::Ret { variant } => writeln!(fmt, "{}: ret {}", label, variant)?,
                Entry::Group { elements } => {
                    write!(fmt, "{}: group ", label)?;
                    for element in elements.iter() {
                        write!(fmt, "{} ", self.find_name(*element)?)?;
                    }
                    writeln!(fmt)?;
                }
                Entry::Extern { .. } => (),
            }
        }
        writeln!(fmt)?;
        for ent in self.entries.iter() {
            if let Entry::Extern { name } = ent {
                writeln!(fmt, "extern {}", name)?;
            }
        }
        writeln!(fmt)?;
        for exp in self.exports.iter() {
            writeln!(fmt, "export {}", exp)?;
        }
        Ok(())
    }
}
impl Debug for PreCompileProgram {
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
impl PreCompileProgram {
    /// merge the instructions from another program
    ///
    pub fn merge(&mut self, other: &PreCompileProgram) -> Result<(), Error> {
        for inst in other.iterate() {
            match inst {
                (_, _, Entry::Extern { .. }) => {}
                (
                    is_export,
                    name,
                    Entry::Call {
                        callee,
                        callcnt,
                        callcont,
                    },
                ) => {
                    let _ = self.define_call(
                        name,
                        other.find_name(*callee)?,
                        *callcnt,
                        other.find_name(*callcont)?,
                    )?;
                    if is_export {
                        self.set_export(name)?
                    }
                }
                (is_export, name, Entry::Group { elements }) => {
                    let mut v = vec![];
                    for element in elements {
                        v.push(other.find_name(*element)?)
                    }
                    let _ = self.define_group(name, &v)?;
                    if is_export {
                        self.set_export(name)?
                    }
                }
                (is_export, name, Entry::Jmp { cont, per }) => {
                    let _ = self.define_jmp(name, other.find_name(*cont)?, per)?;
                    if is_export {
                        self.set_export(name)?
                    }
                }
                (is_export, name, Entry::Ret { variant }) => {
                    let _ = self.define_ret(name, *variant)?;
                    if is_export {
                        self.set_export(name)?
                    }
                }
            }
        }
        Ok(())
    }
    /// Set an entry to be exported
    ///
    pub fn set_export(&mut self, label: impl StringLike) -> Result<(), Error> {
        if self.defined_ent.contains_key(label.as_str()) {
            let _ = self.exports.insert(label.to_string());
            Ok(())
        } else {
            bail!("label not found");
        }
    }
    pub fn delete_ent(&mut self, label: impl StringLike) -> Result<(), Error> {
        let labelent = self.defined_ent.get(label.as_str());
        if let Some(ent) = labelent {
            let ent = *ent;
            *ent.access_mut(self)? = Entry::Extern {
                name: label.to_string(),
            };
        }
        Ok(())
    }
    /// Define a jmp instruction
    ///
    /// A jmp instruction performs a specific permutation
    /// on its values. Then it will jump to another entry.
    ///
    /// label: the name of the entry
    /// cont: the entry this instruction will jump to
    /// per: how to permutate the values
    ///
    pub fn define_jmp(
        &mut self,
        label: impl StringLike,
        cont: impl StringLike,
        per: impl AsPermutation,
    ) -> Result<EntryRef, Error> {
        let cont = self.define_extern_or_entry(cont)?;
        self.define_jmp_internal(label, cont, per.as_permutation()?)
    }
    /// Define a call instruction
    ///
    /// A Call instruction will seperate the current context
    /// into two. One of them will be put in a closure and add to
    /// the other as a single value. Then it will jump to another
    /// entry.
    ///
    /// label: the name of this entry
    /// callee: the entry this entry will jump to
    /// callcnt: how many values the current context will keep
    /// callcont: an entry to keep the rest of context
    ///
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
    /// Define a return instruction
    ///
    /// The first value of the context will be picked up and treat
    /// as a closure. Its captured context released and merged
    /// into the current context. Then the closure's group entry
    /// reference will contain some variants. The specified variant
    /// will be used as the return target.
    ///
    /// variant: which element of the group should be refer to
    ///
    pub fn define_ret(&mut self, label: impl StringLike, variant: u8) -> Result<EntryRef, Error> {
        let ent = Entry::Ret { variant };
        self.define_ent_internal(label, ent)
    }
    /// Define a group of instructions
    ///
    /// A group of entries contains zero of more variants.
    /// When called with a specific variants, it will be picked up
    /// and being used as the execution target.
    ///
    /// label: the name of this group entry
    /// elements: the variant entries
    ///
    pub fn define_group(
        &mut self,
        label: impl StringLike,
        elements: &[impl StringLike],
    ) -> Result<EntryRef, Error> {
        let elements = collect_successful(
            elements
                .iter()
                .map(|e| self.define_extern_or_entry(e.clone_string())),
        )?;
        self.define_group_internal(label, elements)
    }
    /// Returns all defined external entries, including the name
    ///
    pub fn externs(&self) -> Vec<(String, EntryRef)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, e)| match e {
                Entry::Extern { name, .. } => Some((name.clone(), EntryRef::new(index))),
                _ => None,
            })
            .collect()
    }
    /// Compile this program with a set of external functions
    ///
    pub fn compile(&self, externs: impl Iterator<Item = ExternEntry>) -> Result<Program, Error> {
        let mut cm = CodeMap::new();
        let ds = self.dependency_sort();
        let sorted = ds
            .iter()
            .flat_map(|(_, e)| e)
            .collect::<BTreeSet<&EntryRef>>();
        // dependency_sort will put different entries into categories where
        // level 0 are externs, level 1 are returns or only refering to externs,
        // level 2 are only refering level 1 or less, etc.
        //
        // If there is an entry have not been sourt out, this entry must be in a circular reference.
        // We then find sucn entries and report error on those.
        //
        if sorted.len() != self.entries.len() {
            error!("{} entries was found involved in a circular reference without groups (call conts).",
                self.entries.len()-sorted.len());
            for v in self
                .entries
                .iter()
                .enumerate()
                .filter(|(i, _)| !sorted.contains(&EntryRef::new(*i)))
            {
                error!("{}", self.find_name(EntryRef::new(v.0))?);
            }
            bail!("circular reference detected");
        }
        let mut externs_map: HashMap<String, ExternEntry> = HashMap::new();
        for ext in externs {
            let name = ext.name().into();
            externs_map.insert(name, ext);
        }
        // Starting from the lowest level, we add compiled instructions to the compiled program.
        //
        for (_level, entries) in ds {
            for entry in entries {
                let entryref = entry;
                let entry = entry.access(&self)?;
                match entry {
                    Entry::Extern { name } => {
                        if let Some(e) = externs_map.remove(name) {
                            cm.add_extern(entryref, e);
                        } else {
                            bail!("Extern entry not found {}", name);
                        }
                    }
                    Entry::Ret { variant } => {
                        cm.add_return(entryref, *variant);
                    }
                    Entry::Jmp { cont, per } => {
                        cm.add_jmp(entryref, *cont, *per)?;
                    }
                    Entry::Call {
                        callee,
                        callcnt,
                        callcont,
                    } => {
                        cm.add_call(entryref, *callee, *callcnt, *callcont)?;
                    }
                    Entry::Group { elements } => {
                        cm.add_group(entryref, elements)?;
                    }
                }
            }
        }
        // Mark exports
        for export in self.exports.iter() {
            let name = export.clone();
            let ent = self
                .defined_ent
                .get(export)
                .ok_or_else(|| format_err!("Invalid export"))?;
            match ent.access(self) {
                Ok(Entry::Group { .. }) => {
                    cm.add_export_group(*ent, name)?;
                }
                _ => {
                    cm.add_export(*ent, name)?;
                }
            }
        }
        Ok(cm.destruct().0)
    }

    pub(crate) fn entry(&self, idx: usize) -> Result<&Entry, Error> {
        if idx < self.entries.len() {
            Ok(&self.entries[idx])
        } else {
            bail!("Invalid entry ref for PM")
        }
    }
    pub(crate) fn entry_mut(&mut self, idx: usize) -> Result<&mut Entry, Error> {
        if idx < self.entries.len() {
            Ok(&mut self.entries[idx])
        } else {
            bail!("Invalid entry ref for PM")
        }
    }

    fn find_name(&self, entry: EntryRef) -> Result<&str, std::fmt::Error> {
        for e in self.defined_ent.iter() {
            if entry == *e.1 {
                return Ok(&e.0);
            }
        }
        Err(std::fmt::Error::default())
    }
    fn define_ent_internal(
        &mut self,
        label: impl StringLike,
        ent: Entry,
    ) -> Result<EntryRef, Error> {
        let idx = self.entries.len();
        let ret = EntryRef::new(idx);
        let labelent = self.defined_ent.get(label.as_str());
        if let Some(ext) = labelent {
            let ent_orig = format!("{}", ext.access(self)?);
            info!("Redefine {} => {}", ent_orig, ent);
            let ext = *ext;
            *ext.access_mut(self)? = ent;
        } else {
            self.entries.push(ent);
            let _ = self.defined_ent.insert(label.to_string(), ret);
        }
        Ok(ret)
    }
    fn define_extern_or_entry(&mut self, name: impl StringLike) -> Result<EntryRef, Error> {
        if let Some(ent) = self.defined_ent.get(name.as_str()) {
            return Ok(*ent);
        }
        let ent = Entry::Extern {
            name: name.clone_string(),
        };
        let r = self.define_ent_internal(name, ent)?;
        Ok(r)
    }
    fn define_jmp_internal(
        &mut self,
        name: impl StringLike,
        cont: EntryRef,
        per: Permutation,
    ) -> Result<EntryRef, Error> {
        let ent = Entry::Jmp { cont, per };
        self.define_ent_internal(name, ent)
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
        self.define_ent_internal(name, ent)
    }
    fn define_group_internal(
        &mut self,
        name: impl StringLike,
        elements: Vec<EntryRef>,
    ) -> Result<EntryRef, Error> {
        let ent = Entry::Group { elements };
        self.define_ent_internal(name, ent)
    }

    fn find_refs(&self, seed: &BTreeSet<EntryRef>) -> BTreeSet<EntryRef> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, e)| match e {
                Entry::Call {
                    callee, callcont, ..
                } => {
                    if seed.iter().any(|e| *e == *callee)
                        && (callcont.is_group_in(self) || seed.iter().any(|e| *e == *callcont))
                    {
                        Some(EntryRef::new(index))
                    } else {
                        None
                    }
                }
                Entry::Jmp { cont, .. } => {
                    if seed.iter().any(|e| *e == *cont) {
                        Some(EntryRef::new(index))
                    } else {
                        None
                    }
                }
                Entry::Group { elements } => {
                    for element in elements {
                        seed.iter().find(|e| *e == element)?;
                    }
                    Some(EntryRef::new(index))
                }
                Entry::Ret { .. } => Some(EntryRef::new(index)),
                _ => None,
            })
            .collect()
    }
    fn iterate(&self) -> impl Iterator<Item = (bool, &str, &Entry)> {
        struct PIterator<'name>(usize, &'name PreCompileProgram, Vec<EntryRef>);
        impl<'name> Iterator for PIterator<'name> {
            type Item = (bool, &'name str, &'name Entry);
            fn next(&mut self) -> Option<Self::Item> {
                let entref = EntryRef::new(self.0);
                let ent = entref.access(self.1);
                if let Ok(ent) = ent {
                    let is_export = self.2.contains(&entref);
                    let name = if let Ok(name) = self.1.find_name(EntryRef::new(self.0)) {
                        name
                    } else {
                        return None;
                    };
                    self.0 += 1;
                    Some((is_export, name, ent))
                } else {
                    None
                }
            }
        }
        let exps = self
            .exports
            .iter()
            .map(|export| self.defined_ent[export])
            .collect();
        PIterator(0, self, exps)
    }
    fn dependency_sort(&self) -> BTreeMap<usize, BTreeSet<EntryRef>> {
        let mut r = BTreeMap::new();
        let mut s1: BTreeSet<EntryRef> = self.externs().iter().map(|(_, e)| *e).collect();

        let _ = r.insert(0, s1.clone());
        let mut i = 1;
        loop {
            let s2 = self.find_refs(&s1);
            let s2: BTreeSet<EntryRef> = s2.difference(&s1).copied().collect();
            if s2.is_empty() {
                break;
            }
            s1 = s1.union(&s2).copied().collect();
            let _ = r.insert(i, s2);
            i += 1;
        }
        r
    }
}

fn collect_successful<T, E>(i: impl Iterator<Item = Result<T, E>>) -> Result<Vec<T>, E> {
    let mut d = vec![];
    for r in i {
        d.push(r?);
    }
    Ok(d)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_group() {
        let mut p: crate::PreCompileProgram = Default::default();
        let _ = p.define_group("group", &["test"]).unwrap();
        let _ = p.define_call("test", "test", 0, "group").unwrap();
    }
}
