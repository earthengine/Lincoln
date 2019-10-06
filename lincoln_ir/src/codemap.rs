use crate::entry::EntryRef;
use lincoln_common::StringLike;
use lincoln_compiled::{BuildError, CodeRef, ExternEntry, GroupRef, Permutation, Program};
use std::collections::BTreeMap;

#[derive(Fail, Debug)]
pub enum CodeMapError {
    #[fail(display = "Entry not found: {}", _0)]
    EntryNotFound(EntryRef),
    #[fail(display = "Code reference undefined: {}", _0)]
    CodeRefUndefined(EntryRef),
    #[fail(display = "{}", _0)]
    Build(BuildError),
}

pub struct CodeMap {
    coderef_map: BTreeMap<EntryRef, CodeRef>,
    group_map: BTreeMap<EntryRef, GroupRef>,
    prog: Program,
}
impl CodeMap {
    pub fn new() -> Self {
        Self {
            coderef_map: BTreeMap::new(),
            group_map: BTreeMap::new(),
            prog: Default::default(),
        }
    }
    pub(crate) fn add_extern(&mut self, ent: EntryRef, ext: ExternEntry) {
        self.coderef_map.insert(ent, self.prog.add_extern(ext));
    }
    pub(crate) fn add_return(&mut self, ent: EntryRef, variant: u8) {
        self.coderef_map.insert(ent, self.prog.add_return(variant));
    }
    pub(crate) fn add_jmp(
        &mut self,
        ent: EntryRef,
        cont: EntryRef,
        per: Permutation,
    ) -> Result<(), CodeMapError> {
        let cont = *self
            .coderef_map
            .get(&cont)
            .ok_or(CodeMapError::EntryNotFound(cont))?;
        self.coderef_map.insert(ent, self.prog.add_jump(cont, per));
        Ok(())
    }
    pub(crate) fn add_call(
        &mut self,
        ent: EntryRef,
        callee: EntryRef,
        callcnt: u8,
        callcont: EntryRef,
    ) -> Result<(), CodeMapError> {
        let call = *self
            .coderef_map
            .get(&callee)
            .ok_or(CodeMapError::EntryNotFound(callee))?;
        let cont = self.group_map.get(&callcont);
        // The continuation part of a `call` instruction is a group.
        // If the group has been defined, add a new entry to it.
        // Otherwise create a new group and add an entry.
        match cont {
            Some(cont) => {
                self.coderef_map
                    .insert(ent, self.prog.add_call(call, callcnt, *cont));
            }
            None => {
                let grp = self.prog.add_empty_group();
                let _ = self.group_map.insert(callcont, grp);
                if let Some(cont) = self.coderef_map.get(&callcont) {
                    self.prog
                        .add_group_entry(grp, *cont)
                        .map_err(CodeMapError::Build)?;
                }
                self.coderef_map
                    .insert(ent, self.prog.add_call(call, callcnt, grp));
            }
        }
        Ok(())
    }
    pub(crate) fn add_group(
        &mut self,
        ent: EntryRef,
        elements: &[EntryRef],
    ) -> Result<(), CodeMapError> {
        let grp = if let Some(grp) = self.group_map.get(&ent) {
            debug!("group inserted {:?}", grp);
            *grp
        } else {
            let grp = self.prog.add_empty_group();
            self.group_map.insert(ent, grp);
            debug!("new group {:?}", grp);
            grp
        };
        for element in elements {
            let element = self
                .coderef_map
                .get(element)
                .ok_or(CodeMapError::EntryNotFound(*element))?;
            self.prog
                .add_group_entry(grp, element.clone())
                .map_err(CodeMapError::Build)?;
        }
        self.group_map.insert(ent, grp);
        Ok(())
    }
    pub(crate) fn add_export_group(
        &mut self,
        ent: EntryRef,
        name: impl StringLike,
    ) -> Result<(), CodeMapError> {
        let grp = self
            .group_map
            .get(&ent)
            .ok_or(CodeMapError::EntryNotFound(ent))?;
        self.prog.add_export(name, *grp);
        Ok(())
    }
    pub(crate) fn add_export(
        &mut self,
        ent: EntryRef,
        name: impl StringLike,
    ) -> Result<(), CodeMapError> {
        let grp = self.prog.add_empty_group();
        let ent = self
            .coderef_map
            .get(&ent)
            .ok_or(CodeMapError::CodeRefUndefined(ent))?;
        self.prog
            .add_group_entry(grp, ent.clone())
            .map_err(CodeMapError::Build)?;
        self.prog.add_export(name, grp);
        Ok(())
    }
    pub(crate) fn destruct(
        self,
    ) -> (
        Program,
        BTreeMap<EntryRef, CodeRef>,
        BTreeMap<EntryRef, GroupRef>,
    ) {
        (self.prog, self.coderef_map, self.group_map)
    }
}
