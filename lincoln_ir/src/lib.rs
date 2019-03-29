#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod codemap;
mod entry;
mod program;

#[cfg(test)]
mod tests;

pub use entry::{Entry, EntryRef};
pub use program::PreCompileProgram;
