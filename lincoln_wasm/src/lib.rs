#[macro_use]
extern crate log;
extern crate console_log;
extern crate js_sys;
extern crate web_sys;

mod lincoln_int;
mod lincoln_jsvalue;
mod lincoln_context;

use lincoln_jsvalue::JsResult;
use wasm_bindgen::prelude::*;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    if cfg!(debug_assertions) {
        console_log::init_with_level(log::Level::Debug).map_err_js()?;
    } else {
        console_log::init_with_level(log::Level::Info).map_err_js()?;
    }
    Ok(())
}
