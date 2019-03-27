#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

use core::fmt::{Debug, Display, Formatter};
use failure::Error;
use lincoln_common::traits::StringLike;
use lincoln_common::traits::{Access, AccessMut};
use lincoln_compiled::{AsPermutation, ExternEntry, GroupRef, Permutation, Program};
use std::collections::{BTreeMap, BTreeSet};

mod entry;

pub use entry::{Entry, EntryRef};

///
#[derive(Serialize, Deserialize, Default)]
pub struct PreCompileProgram {
    defined_ent: BTreeMap<String, EntryRef>,
    entries: Vec<Entry>,
    exports: BTreeSet<String>,
}
impl Display for PreCompileProgram {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        for (idx, ent) in self.entries.iter().enumerate() {
            let label = self.find_name(&EntryRef::new(idx))?;
            match ent {
                Entry::Jmp { cont, per } => {
                    write!(fmt, "{}: jmp {} #!{}\n", label, self.find_name(cont)?, per)?
                }
                Entry::Call {
                    callee,
                    callcnt,
                    callcont,
                } => write!(
                    fmt,
                    "{}: call {} {} {}\n",
                    label,
                    self.find_name(callee)?,
                    callcnt,
                    self.find_name(callcont)?
                )?,
                Entry::Ret { variant } => write!(fmt, "{}: ret {}\n", label, variant)?,
                Entry::Group { elements } => {
                    write!(fmt, "{}: group ", label)?;
                    for element in elements.iter() {
                        write!(fmt, "{} ", self.find_name(element)?)?;
                    }
                    writeln!(fmt)?;
                }
                Entry::Extern { .. } => (),
            }
        }
        writeln!(fmt, "")?;
        for ent in self.entries.iter() {
            match ent {
                Entry::Extern { name } => {
                    writeln!(fmt, "extern {}", name)?;
                }
                _ => (),
            }
        }
        writeln!(fmt, "")?;
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
                        other.find_name(callee)?,
                        *callcnt,
                        other.find_name(callcont)?,
                    )?;
                    if is_export {
                        self.set_export(name)?
                    }
                }
                (is_export, name, Entry::Group { elements }) => {
                    let mut v = vec![];
                    for element in elements {
                        v.push(other.find_name(element)?)
                    }
                    let _ = self.define_group(name, &v)?;
                    if is_export {
                        self.set_export(name)?
                    }
                }
                (is_export, name, Entry::Jmp { cont, per }) => {
                    let _ = self.define_jmp(name, other.find_name(cont)?, per)?;
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
        if self.defined_ent.contains_key(label.as_ref()) {
            let _ = self.exports.insert(label.into());
            Ok(())
        } else {
            bail!("label not found");
        }
    }
    pub fn delete_ent(&mut self, label: impl StringLike) {
        let labelent = self.defined_ent.get(label.as_ref());
        if let Some(ent) = labelent {
            if ent.is_in(self) {
                let ent = *ent;
                *ent.access_mut(self).unwrap() = Entry::Extern { name: label.into() };
            }
        }
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
        self.define_ret_internal(label, variant)
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
    pub fn compile(&self, externs: impl AsRef<[fn() -> ExternEntry]>) -> Result<Program, Error> {
        let mut prog: Program = Default::default();
        let mut coderef_map = BTreeMap::new();
        let mut groupdef_map: BTreeMap<EntryRef, GroupRef> = BTreeMap::new();
        let ds = self.dependency_sort();
        let sorted = ds
            .iter()
            .flat_map(|(_, e)| e)
            .collect::<BTreeSet<&EntryRef>>();
        if sorted.len() != self.entries.len() {
            error!("{} entries was found involved in a circular reference without groups (call conts).",
                self.entries.len()-sorted.len());
            for v in self
                .entries
                .iter()
                .enumerate()
                .filter(|(i, _)| !sorted.contains(&EntryRef::new(*i)))
            {
                error!("{}", self.find_name(&EntryRef::new(v.0))?);
            }
            bail!("circular reference detected");
        }
        for (level, entries) in ds {
            for entry in entries {
                let entryref = entry;
                let entry = entry
                    .access(&self)
                    .ok_or(format_err!("Invalid entry ref for PM"))?;
                match entry {
                    Entry::Extern { name } => {
                        if let Some(e) = externs.as_ref().iter().find(|e| (*e)().name() == name) {
                            let e = (*e).clone();
                            let _ = coderef_map.insert(entryref, prog.add_extern(e()));
                            debug!("define extern {}", name);
                        } else {
                            bail!("Extern entry not found {}", name);
                        }
                    }
                    Entry::Ret { variant } => {
                        let _ = coderef_map.insert(entryref, prog.add_return(*variant));
                        debug!("define return {}", self.find_name(&entryref)?);
                    }
                    Entry::Jmp { cont, per } => {
                        let cont = coderef_map.get(&cont).ok_or(format_err!(
                            "Dependency error: cont for Jmp is undefined: {}, {}, {}",
                            cont,
                            level,
                            self.find_name(&entryref)?
                        ))?;
                        let _ = coderef_map.insert(entryref, prog.add_jump(cont.clone(), *per));
                        debug!("define jump {}", self.find_name(&entryref)?);
                    }
                    Entry::Call {
                        callee,
                        callcnt,
                        callcont,
                    } => {
                        let call = coderef_map.get(&callee).ok_or(format_err!(
                            "Dependency error: callee for Call is undefined"
                        ))?;
                        let cont = groupdef_map.get(callcont);
                        match cont {
                            Some(cont) => {
                                let _ = coderef_map
                                    .insert(entryref, prog.add_call(call.clone(), *callcnt, *cont));
                                debug!("define call {}", self.find_name(&entryref)?);
                            }
                            None => {
                                let grp = prog.add_empty_group();
                                let _ = groupdef_map.insert(*callcont, grp);
                                if let Some(cont) = coderef_map.get(&callcont) {
                                    prog.add_group_entry(grp, cont.clone())?;
                                }
                                let _ = coderef_map
                                    .insert(entryref, prog.add_call(call.clone(), *callcnt, grp));
                                debug!("define call {} for group", self.find_name(&entryref)?);
                            }
                        }
                    }
                    Entry::Group { elements } => {
                        debug!("defining group {}...", self.find_name(&entryref)?);
                        let grp = if let Some(grp) = groupdef_map.get(&entryref) {
                            debug!("group inserted {:?}", grp);
                            *grp
                        } else {
                            let grp = prog.add_empty_group();
                            let _ = groupdef_map.insert(entryref, grp);
                            debug!("new group {:?}", grp);
                            grp
                        };
                        for element in elements {
                            let element = coderef_map.get(element).ok_or(format_err!(
                                "Deoendency error: group element is not defined"
                            ))?;
                            prog.add_group_entry(grp, element.clone())?;
                        }
                        let _ = groupdef_map.insert(entryref, grp);
                    }
                }
            }
        }
        for export in self.exports.iter() {
            let name = export.clone();
            let ent = self
                .defined_ent
                .get(export)
                .ok_or(format_err!("Invalid export"))?;
            match ent.access(self) {
                Some(Entry::Group { .. }) => {
                    let grp = groupdef_map
                        .get(ent)
                        .ok_or(format_err!("group not found"))?;
                    prog.add_export(name, *grp)
                }
                _ => {
                    let grp = prog.add_empty_group();
                    let ent = coderef_map.get(ent).ok_or(format_err!(
                        "entry not found {}({})",
                        ent,
                        export
                    ))?;
                    prog.add_group_entry(grp, ent.clone())?;
                    prog.add_export(name, grp)
                }
            }
        }
        Ok(prog)
    }

    fn find_refs(&self, seed: &BTreeSet<EntryRef>) -> BTreeSet<EntryRef> {
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
                        Some(EntryRef::new(index))
                    } else {
                        None
                    }
                }
                Entry::Jmp { cont, .. } => {
                    if let Some(_) = seed.iter().find(|e| *e == cont) {
                        Some(EntryRef::new(index))
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
                    Some(EntryRef::new(index))
                }
                Entry::Ret { .. } => Some(EntryRef::new(index)),
                _ => None,
            })
            .collect()
    }
    fn iterate<'name>(&'name self) -> impl Iterator<Item = (bool, &'name str, &'name Entry)> {
        struct PIterator<'name>(usize, &'name PreCompileProgram, Vec<EntryRef>);
        impl<'name> Iterator for PIterator<'name> {
            type Item = (bool, &'name str, &'name Entry);
            fn next(&mut self) -> Option<Self::Item> {
                let entref = EntryRef::new(self.0);
                let ent = entref.access(self.1);
                if let Some(ent) = ent {
                    let is_export = self.2.contains(&entref);
                    let name = if let Ok(name) = self.1.find_name(&EntryRef::new(self.0)) {
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

    fn find_name(&self, entry: &EntryRef) -> Result<&str, std::fmt::Error> {
        for e in self.defined_ent.iter() {
            if entry == e.1 {
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
        let labelent = self.defined_ent.get(label.as_ref());
        if let Some(ext) = labelent {
            let ent_orig = format!("{}", ext.access(self).unwrap());
            info!("Redefine {} => {}", ent_orig, ent);
            let ext = *ext;
            //if let Entry::Extern { .. } = ent_orig {
            *ext.access_mut(self).unwrap() = ent;
        //} else {
        //bail!("Redefine entry that is not extern: {} => {}", ent_orig, ent);

        //}
        } else {
            self.entries.push(ent);
            let _ = self.defined_ent.insert(label.into(), ret);
        }
        Ok(ret)
    }
    fn define_extern_or_entry(&mut self, name: impl StringLike) -> Result<EntryRef, Error> {
        if let Some(ent) = self.defined_ent.get(name.as_ref()) {
            return Ok(*ent);
        }
        let ent = Entry::Extern {
            name: name.clone_string(),
        };
        let r = self.define_ent_internal(name.into(), ent)?;
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
    fn dependency_sort(&self) -> BTreeMap<usize, BTreeSet<EntryRef>> {
        let mut r = BTreeMap::new();
        let mut s1: BTreeSet<EntryRef> = self.externs().iter().map(|(_, e)| *e).collect();

        let _ = r.insert(0, s1.clone());
        let mut i = 1;
        loop {
            let s2 = self.find_refs(&s1);
            let s2: BTreeSet<EntryRef> = s2.difference(&s1).map(|e| *e).collect();
            if s2.len() == 0 {
                break;
            }
            s1 = s1.union(&s2).map(|e| *e).collect();
            let _ = r.insert(i, s2);
            i = i + 1;
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
        let mut p: crate::pre_compile::PreCompileProgram = Default::default();
        let _ = p.define_group("group", &["test"]).unwrap();
        let _ = p.define_call("test", "test", 0, "group").unwrap();
    }
}
