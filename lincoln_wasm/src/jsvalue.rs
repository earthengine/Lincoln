use wasm_bindgen::JsValue;
use lincoln_compiled::{Value, Program, Context, CodeRef};
use failure::Error;

#[derive(Debug)]
pub(crate) struct JsValueWrap {
    pub value: JsValue
}

impl Value for JsValueWrap {
    fn eval(self: Box<Self>, _: &Program, _: Context, _: u8)
        -> Result<(CodeRef, Context), Error> {
        bail!("not implemented")
    }
    fn into_wrapped(self: Box<Self>, _: &Program) -> Result<Box<dyn Value>, Error> {
        Ok(lincoln_compiled::wrap(self.value))
    }
}