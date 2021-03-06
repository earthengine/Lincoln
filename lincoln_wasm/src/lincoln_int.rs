use js_sys::{try_iter, Array, Function, JsString, Reflect};
use wasm_bindgen::prelude::*;

use lincoln_common::traits::Access;
use lincoln_compiled::{CodeRef, Context, EvalFn, ExternEntry, Permutation, Program, ValueFn};
use lincoln_ir::PreCompileProgram;

use crate::lincoln_jsvalue::{
    eval_function, unwrap_jsvalue, wrap_jsvalue, CollapseResult, JsResult,
};

fn jsvalue_to_string(v: JsValue) -> String {
    let s: JsString = v.into();
    s.into()
}

#[wasm_bindgen]
pub struct LincolnIntepretor {
    program: PreCompileProgram,
    compiled: Option<Program>,
    context: Option<Context>,
    current: Option<CodeRef>,
    round: usize,
}
impl Default for LincolnIntepretor {
    fn default() -> Self {
        LincolnIntepretor {
            compiled: None,
            context: None,
            current: None,
            round: 0,
            program: PreCompileProgram::default(),
        }
    }
}
#[wasm_bindgen]
impl LincolnIntepretor {
    pub fn new() -> LincolnIntepretor {
        Default::default()
    }
    pub fn set_program(&mut self, prog: &JsValue) -> Result<(), JsValue> {
        let prog: PreCompileProgram = prog.into_serde().map_err_js()?;
        debug!("{}", prog);
        self.program = prog;
        Ok(())
    }
    pub fn get_program(&mut self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.program).map_err_js()
    }
    pub fn jmp(&mut self, jmplabel: &str, jmpcont: &str, per: u32) -> Result<(), JsValue> {
        let pm = &mut self.program;
        let per: Permutation = u64::from(per).into();
        pm.define_jmp(jmplabel, jmpcont, per)
            .map(|e| debug!("{:?}", e.access(&pm)))
            .map_err_js()
    }
    pub fn call(
        &mut self,
        calllabel: &str,
        callee: &str,
        callcnt: u8,
        callcont: &str,
    ) -> Result<(), JsValue> {
        let pm = &mut self.program;
        pm.define_call(calllabel, callee, callcnt, callcont)
            .map(|e| debug!("{:?}", e.access(&pm)))
            .map_err_js()
    }
    pub fn ret(&mut self, retlabel: &str, variant: u8) -> Result<(), JsValue> {
        let pm = &mut self.program;
        pm.define_ret(retlabel, variant)
            .map(|e| debug!("{:?}", e.access(&pm)))
            .map_err_js()
    }
    pub fn group(&mut self, grouplabel: &str, elements: Box<[JsValue]>) -> Result<(), JsValue> {
        let elements: Vec<String> = elements.iter().filter_map(JsValue::as_string).collect();
        let pm = &mut self.program;
        pm.define_group(grouplabel, &elements)
            .map(|e| debug!("{:?}", e.access(&pm)))
            .map_err_js()
    }
    pub fn set_export(&mut self, exportlabel: &str) -> Result<(), JsValue> {
        let pm = &mut self.program;
        pm.set_export(exportlabel).map_err_js()
    }
    pub fn delete(&mut self, deletelabel: &str) -> Result<(), JsValue> {
        let pm = &mut self.program;
        pm.delete_ent(deletelabel).map_err_js()
    }
    pub fn compile(&mut self, externs: &JsValue) -> Result<(), JsValue> {
        let pm = &mut self.program;
        let value = JsValue::from("Not iterable");
        let iter = try_iter(externs)?.ok_or(value)?;
        let mut exts = vec![];
        for v in iter {
            let v = v.unwrap_or_else(|e| e);
            if v.is_function() {
                let f: Function = v.into();
                let name: String = f.name().into();
                debug!("external Function - name: {}", name);
                let eval = EvalFn::Dyn(Box::new(move |ctx| eval_function(&f, ctx)));
                let ext = ExternEntry::Eval { name, eval };
                exts.push(ext);
            } else if let (Some(name), value) = (
                Reflect::get(&v, &"name".into())
                    .unwrap_or_else(|e| e)
                    .as_string(),
                Reflect::get(&v, &"value".into()).unwrap_or_else(|e| e),
            ) {
                debug!("external Value - name: {}", name);
                let value = ValueFn::Dyn(Box::new(move || wrap_jsvalue(&value)));
                let ext = ExternEntry::Value { name, value };
                exts.push(ext);
            }
        }

        let compiled = pm.compile(exts.into_iter()).map_err_js()?;
        debug!("compiled: {:?}", compiled);
        self.compiled = Some(compiled);

        Ok(())
    }
    pub fn run(
        &mut self,
        entry: &str,
        variant: u8,
        values: &JsValue,
        step: bool,
    ) -> Result<(), JsValue> {
        let value_iter = js_sys::try_iter(values)?.ok_or_else(|| JsValue::from("not iterable"))?;
        let compiled = if let Some(compiled) = &mut self.compiled {
            compiled
        } else {
            return Err("Not compiled".into());
        };
        let mut ctx = Context::default();
        ctx.push(lincoln_compiled::native_closure("done", |_, v| {
            debug!("Terminate at variant {}", v);
            Ok(CodeRef::Termination)
        }));
        for (i, value) in value_iter.enumerate() {
            let value = value.collapse();
            debug!("value {} = {}", i, jsvalue_to_string(value.clone()));
            ctx.push(wrap_jsvalue(&value));
        }
        if !step {
            compiled.run(&mut ctx, entry, variant, None).map_err_js()?;
            self.context = Some(ctx);
            Ok(())
        } else {
            let entry = compiled.get_export_ent(entry, variant).map_err_js()?;
            debug!("{} {}", entry, ctx);
            self.context = Some(ctx);
            self.current = Some(entry);
            self.round = 1;
            Ok(())
        }
    }
    pub fn step(&mut self) -> Result<bool, JsValue> {
        if let (Some(compiled), Some(current), Some(context)) =
            (&mut self.compiled, self.current, &mut self.context)
        {
            let next = compiled.eval(context, &current).map_err_js()?;
            self.round += 1;
            if let CodeRef::Termination = next {
                self.current = None;
                //self.context = None;
                self.round = 0;
                Ok(false)
            } else {
                debug!("{}: {} {}", self.round, next, context);
                self.current = Some(next);
                Ok(true)
            }
        } else {
            Err("Stepping on wrong state".into())
        }
    }
    pub fn get_context(&mut self) -> Result<JsValue, JsValue> {
        debug!("get_context");
        let result = Array::new();
        if let Some(ctx) = &mut self.context {
            for value in ctx.iterate() {
                let value = unwrap_jsvalue(value).map_err_js()?;
                result.unshift(&value);
            }
        }
        Ok(result.into())
    }
}
