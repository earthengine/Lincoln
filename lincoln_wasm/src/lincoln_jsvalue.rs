use lincoln_compiled::{Context, Value, CodeRef, EvalError, ValueAccessError, wrap};

use std::fmt::{Formatter, Display, Debug};

use js_sys::{Array, Function, Reflect};
use wasm_bindgen::prelude::*;

pub trait JsResult<T> {
    fn map_err_js(self) -> Result<T,JsValue>;
}
impl<T,E> JsResult<T> for Result<T,E>
where E: Display
{
    fn map_err_js(self) -> Result<T,JsValue> {
        self.map_err(|e| format!("{}", e).into())
    }
}
pub trait CollapseResult<T> {
    fn collapse(self) -> T;
}
impl<T> CollapseResult<T> for Result<T,T>
{
    fn collapse(self) -> T {
        self.unwrap_or_else(|e| e)
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct LincolnJsValue(JsValue);
impl Debug for LincolnJsValue {
    fn fmt(&self, fmt:&mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}

impl Display for LincolnJsValue {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        let r:Reflect = self.0.clone().into();
        let s:String = r.to_string().into();
        write!(fmt, "[{}]", s)
    }
}

impl Value for LincolnJsValue {
    fn eval(self: Box<Self>, _ctx: &mut Context, _variant: u8) -> Result<CodeRef, EvalError> {
        Err(EvalError::CallingWrapped)
    }
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>> {
        Some(wrap(self))
    }
}
pub fn wrap_jsvalue(value:&JsValue) -> Box<dyn Value> {
    Box::new(LincolnJsValue(value.clone()))
}
pub fn unwrap_jsvalue(value: Box<Value>) -> Result<JsValue, EvalError> {
    Ok(value.into_boxed_any()
        .downcast::<LincolnJsValue>()
        .map_err(|e| ValueAccessError::UnwrapNotWrapped(format!("{:?}", e)))?
        .0)
}

pub fn eval_function(f: &Function, ctx: &mut Context) -> Result<CodeRef, EvalError> {
    let r = ctx.pop()?;
    let args = Array::new();
    while let Ok(v) = ctx.pop() {
        args.unshift(&unwrap_jsvalue(v)?);
    }
    match f.apply(&JsValue::null(), &args) {
        Ok(v) => {
            let value_iter = js_sys::try_iter(&v);
            if let Ok(Some(value_iter)) = value_iter {
                for value in value_iter {
                    let value = value.collapse();
                    ctx.push(wrap_jsvalue(&value));            
                }
            } else {
                ctx.push(wrap_jsvalue(&v));
            }
            r.eval(ctx, 0)
        },
        Err(e) => {
            let value_iter = js_sys::try_iter(&e);
            if let Ok(Some(value_iter)) = value_iter {
                for value in value_iter {
                    let value = value.collapse();
                    ctx.push(wrap_jsvalue(&value));            
                }
            } else {
                ctx.push(wrap_jsvalue(&e));
            }
            r.eval(ctx, 1)
        }
    }        
}
