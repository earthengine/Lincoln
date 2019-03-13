use crate::jsvalue::JsValueWrap;
use lincoln_compiled::Context;
use js_sys::Function;
use crate::externs::FACT_EXTERNS;
use failure::Error;
use lincoln_compiled::{AsPermutation, CodeRef, GroupRef, Program, ExternEntry};
use wasm_bindgen::prelude::*;
use js_sys::{Object, Reflect, Array};

#[wasm_bindgen]
pub struct WasmProgram {
    program: *mut Program,
}
impl Drop for WasmProgram {
    fn drop(&mut self) {
        if !self.program.is_null() {
            unsafe {
                let _ = Box::from_raw(self.program);
            };
        }
    }
}
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub enum WasmCodeRefType {
    Entry,
    Extern,
    Termination
}
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmCodeRef {
    r#type: WasmCodeRefType,
    idx: Option<u32>
}
#[wasm_bindgen]
impl WasmCodeRef {
    pub fn new_entry(entry: u32) -> Self {
        WasmCodeRef { r#type: WasmCodeRefType::Entry,idx: Some(entry)}
    }
    pub fn new_extern(ext: u32) -> Self {
        WasmCodeRef { r#type: WasmCodeRefType::Extern,idx: Some(ext)}
    }
    pub fn new_termination() -> Self {
        WasmCodeRef { r#type: WasmCodeRefType::Termination,idx: None}
    }
}
impl From<WasmCodeRef> for CodeRef {
    fn from(cr:WasmCodeRef) -> Self {
        match cr {
            WasmCodeRef{r#type: WasmCodeRefType::Entry,idx: Some(entry)} => CodeRef::entry(entry as usize),
            WasmCodeRef{r#type: WasmCodeRefType::Extern,idx: Some(ext)} => CodeRef::ext(ext as usize),
            WasmCodeRef{r#type: WasmCodeRefType::Termination,idx: None} => CodeRef::Termination,
            _ => unreachable!()
        }
    }
}
impl From<CodeRef> for WasmCodeRef {
    fn from(cr: CodeRef) -> Self {
        match cr {
            CodeRef::Entry(entry) => WasmCodeRef::new_entry(entry.0 as u32),
            CodeRef::Extern(ext) => WasmCodeRef::new_extern(ext.0 as u32),
            CodeRef::Termination => WasmCodeRef::new_termination()
        }
    }
}

#[wasm_bindgen]
pub struct WasmGroupRef {
    group: u32
}
impl WasmGroupRef {
    pub fn new(idx: u32) -> Self {
        WasmGroupRef { group: idx }
    }
}
impl From<WasmGroupRef> for GroupRef {
    fn from(gr: WasmGroupRef) -> Self {
        GroupRef::new(gr.group as usize)
    }
}
impl From<GroupRef> for WasmGroupRef {
    fn from(gr: GroupRef) -> Self {
        WasmGroupRef { group: gr.get_index() as u32 }
    }
}

#[wasm_bindgen]
pub struct WasmProgramRef {
    program: *const Program
}
impl WasmProgramRef {
    fn as_program(&self) -> &Program {
        unsafe { &*self.program }
    }
}
impl From<&Program> for WasmProgramRef {
    fn from(p:&Program) -> Self {
        WasmProgramRef{program: p as *const Program}
    }
}
impl From<&WasmProgramRef> for JsValue {
    fn from(p: &WasmProgramRef) -> Self {
        (p.program as usize as u32).into()
    }
}

#[wasm_bindgen]
impl WasmProgram {
    pub fn new() -> WasmProgram {
        Box::new(Program::new()).into()
    }
    pub fn add_return(&mut self, variant: u8) -> Result<JsValue, JsValue> {
        coderef_to_jsvalue(self.as_program_mut().add_return(variant))
    }
    pub fn add_jump(&mut self, cont: WasmCodeRef, permutation:String) -> Result<JsValue,JsValue> {
        let cont = cont.into();
        let per = permutation.as_permutation().map_err(|e| JsValue::from(format!("{}", e)))?;
        coderef_to_jsvalue(self.as_program_mut().add_jump(cont, per))
    }
    pub fn add_call(&mut self, callee: WasmCodeRef, num_args: u8, cont: WasmGroupRef) -> Result<JsValue, JsValue> {
        let callee = callee.into();
        let cont = cont.into();
        coderef_to_jsvalue(self.as_program_mut().add_call(callee, num_args, cont))
    }
    pub fn add_empty_group(&mut self) -> Result<JsValue,JsValue> {
        groupref_to_jsvalue(self.as_program_mut().add_empty_group())
    }
    pub fn add_group_entry(&mut self, grp: WasmGroupRef, ent: WasmCodeRef) -> Result<(), JsValue> {
        let grp = grp.into();
        let ent = ent.into();
        self.as_program_mut().add_group_entry(grp, ent).map_err(|e| format!("{}", e).into())
    }
    pub fn add_extern_fn(&mut self, name: String, f: Function) -> Result<JsValue, JsValue> {
        coderef_to_jsvalue(self.as_program_mut().add_extern(
            ExternEntry::Eval{ name: name, eval: Box::new(move |p,c| call_function(&f, &p.into(), c)) }
        ))
    }
    pub fn add_extern_val(&mut self, name: String, f: Function) -> Result<JsValue, JsValue> {
        coderef_to_jsvalue(self.as_program_mut().add_extern(ExternEntry::Value{
            name: name, value: Box::new(move || Box::new(JsValueWrap{value:f.call0(&JsValue::NULL).unwrap()}))
        }))
    }
}

impl From<Box<Program>> for WasmProgram {
    fn from(prog: Box<Program>) -> Self {
        WasmProgram {
            program: Box::into_raw(prog),
        }
    }
}

fn call_function(f: &Function, p:&WasmProgramRef, mut c: Context) -> Result<(CodeRef, Context), Error> {
    let cont = c.pop()?;
    let values:Vec<_> = c.iterate().map(|v| v.into_boxed_any().downcast::<JsValueWrap>()).collect();
    if values.iter().any(|v| v.is_err()) { bail!("Not all values are JsValue") }
    let v = Array::new();
    for value in values.into_iter().map(|v| v.unwrap().value) {
        v.push(&value);
    };
    let a = v.into();
    let r = f.apply(&p.into(), &a).map_err(|e| format_err!("{:?}", e))?;
    let mut c = Context::default();
    c.push(Box::new(JsValueWrap{value:r}));
    cont.eval(p.as_program(), c, 0)
}

fn groupref_to_jsvalue(gr: GroupRef) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    Reflect::set(&obj, &"group".into(), &(gr.get_index() as u32).into())?;
    Ok(obj.into())
}
fn coderef_to_jsvalue(cr: CodeRef) -> Result<JsValue, JsValue> {
    let cr:WasmCodeRef = cr.into();
    JsValue::from_serde(&cr).map_err(|_| "invalid code reference".into())
}

impl WasmProgram {
    pub fn get_program(mut self) -> Box<Program> {
        let r = unsafe { Box::from_raw(self.program) };
        self.program = std::ptr::null_mut();
        r
    }
    pub fn as_program_mut(&self) -> &mut Program {
        unsafe { &mut *self.program }
    }
    pub fn test_prog_internal(&self) -> Result<JsValue, Error> {
        let prog = self.as_program_mut();
        for entry in FACT_EXTERNS {
            prog.add_extern(entry());
        }
        for _i in 0..11 {
            prog.add_empty_group();
        }
        prog.add_return(0);
        prog.add_call(CodeRef::ext(2), 2, GroupRef::new(0));
        prog.add_return(1);
        prog.add_jump(CodeRef::ext(6), "acb".as_permutation()?);
        prog.add_jump(CodeRef::entry(0), "ba".as_permutation()?);
        prog.add_jump(CodeRef::entry(1), "aecdb".as_permutation()?);
        prog.add_jump(CodeRef::entry(2), "ba".as_permutation()?);
        prog.add_jump(CodeRef::entry(0), "cba".as_permutation()?);
        prog.add_call(CodeRef::ext(1), 1, GroupRef::new(1));
        prog.add_call(CodeRef::ext(4), 1, GroupRef::new(2));
        prog.add_call(CodeRef::ext(7), 2, GroupRef::new(3));
        prog.add_jump(CodeRef::entry(8), "ba".as_permutation()?);
        prog.add_call(CodeRef::ext(6), 1, GroupRef::new(4));
        prog.add_jump(CodeRef::entry(10), "ba".as_permutation()?);
        prog.add_call(CodeRef::entry(11), 3, GroupRef::new(5));
        prog.add_call(CodeRef::ext(4), 1, GroupRef::new(6));
        prog.add_call(CodeRef::ext(5), 2, GroupRef::new(8));
        prog.add_call(CodeRef::entry(14), 2, GroupRef::new(8));
        prog.add_jump(CodeRef::entry(15), "cba".as_permutation()?);
        prog.add_jump(CodeRef::entry(16), "aecdb".as_permutation()?);
        prog.add_call(CodeRef::ext(1), 1, GroupRef::new(9));
        prog.add_jump(CodeRef::entry(20), "cba".as_permutation()?);
        prog.add_call(CodeRef::entry(21), 3, GroupRef::new(2));

        prog.add_group_entry(GroupRef::new(0), CodeRef::entry(18))?;
        prog.add_group_entry(GroupRef::new(0), CodeRef::entry(22))?;
        prog.add_group_entry(GroupRef::new(1), CodeRef::entry(5))?;
        prog.add_group_entry(GroupRef::new(2), CodeRef::ext(3))?;
        prog.add_group_entry(GroupRef::new(3), CodeRef::entry(3))?;
        prog.add_group_entry(GroupRef::new(4), CodeRef::entry(9))?;
        prog.add_group_entry(GroupRef::new(5), CodeRef::entry(0))?;
        prog.add_group_entry(GroupRef::new(6), CodeRef::entry(12))?;
        prog.add_group_entry(GroupRef::new(7), CodeRef::entry(13))?;
        prog.add_group_entry(GroupRef::new(8), CodeRef::entry(17))?;
        prog.add_group_entry(GroupRef::new(8), CodeRef::entry(14))?;
        prog.add_group_entry(GroupRef::new(9), CodeRef::entry(19))?;
        prog.add_group_entry(GroupRef::new(10), CodeRef::entry(17))?;

        Ok(0.into())
    }
}
