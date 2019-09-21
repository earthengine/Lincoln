#![deny(bare_trait_objects)]
#![deny(unused_results)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate failure;

pub mod command;
pub mod externs;

use crate::command::{commands, process, CommandContext};
use failure::Error;
use rustyline::Editor;

fn print_help() {
    println!(
        r#"Syntax (<label> are identifiers, [number:u8] are numbers): 
    <jmplabel>: jmp <jmpcont> #!<permutation>
    <calllabel>: call <callee> [callcnt:u8] <callcont>
    <retlabel>: ret [variant:u8]
    <grouplabel>: group <element>*
    setexport <label>
    show external set
    show program
    compile <enternal set>
    run <entry> [variant:u8] "<value>"
    run <entry> [variant:u8] "<value>" step
    save <filename>
    load <filename>
    exit
    
Permutations are strings contains charactor a-t to specify permutations. Examples:

    "" or "a" or "ab" or "abc"... - the identical permutation;
    "ba" or "bac" or "bacd" ... - swap the first element and the second;
    "bac" or "bacd" ... - swap the first element and the third;
    "bca" or "bcad" ... - replace the first with the second, the second with the third, and the
    third with the first.

and so on.
"#
    );
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let commands = commands();

    let mut cmdctx = CommandContext::default();
    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").unwrap_or(());
    loop {
        let line = rl.readline("Lincoln> ")?;
        let l: &str = &line;
        let _ = rl.add_history_entry(l);
        rl.save_history("history.txt")?;

        if let Some(c) = commands.captures(&line) {
            match process(c, &mut cmdctx) {
                Ok(true) => continue,
                Ok(false) => return Ok(()),
                Err(e) => error!("{}", e),
            }
        } else {
            print_help();
        }
    }
}
