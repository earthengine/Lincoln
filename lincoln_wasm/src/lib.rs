#[macro_use]
extern crate failure;
extern crate wasm_bindgen;
#[macro_use]
extern crate serde_derive;

mod externs;
mod wasm_program;
mod jsvalue;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    pub fn zero() -> u32;
    pub fn copy_int(n: u32) -> Box<[u32]>;
    pub fn eq(n1: u32, n2: u32) -> bool;
    pub fn one() -> u32;
    pub fn drop_int(n: u32);
    pub fn minus(n1: u32, n2: u32) -> u32;
    pub fn mul(n1: u32, n2: u32) -> u32;
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello WASM");
}
