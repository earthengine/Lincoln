#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod entry;
mod program;

pub use entry::{Entry, EntryRef};
pub use program::PreCompileProgram;


