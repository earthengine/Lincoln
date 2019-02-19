#![deny(bare_trait_objects)]
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate failure;
extern crate failure_derive;
extern crate regex;
#[macro_use]
extern crate smallvec;

mod coderef;
mod fact_externs;
mod permutation;
mod program;
mod program_manager;
mod value;

use crate::fact_externs::print;
use crate::coderef::Access;
use crate::value::{Context, Value};
use failure::Error;
use program_manager::ProgramManager;
use regex::{Captures, Regex};
use std::fs::File;

use std::io::{stdin, stdout, Write, Read};

fn commands() -> Regex {
    Regex::new(concat!(
        // <jmplabel> jmp <jmpcont> #!<permutation>
        r"(?P<jmp>((?P<jmplabel>\p{XID_Start}\p{XID_Continue}*):\s+",
        r"jmp\s+(?P<jmpcont>\p{XID_Start}\p{XID_Continue}*)\s*(?P<per>#![a-t]{0,20})))\s*$|",
        // <calllabel> call <callee> [callcnt] <callcont>
        r"(?P<call>((?P<calllabel>\p{XID_Start}\p{XID_Continue}*):\s+",
        r"call\s+(?P<callee>\p{XID_Start}\p{XID_Continue}*)\s+(?P<callcnt>([1-9][0-9]*|0))\s+",
        r"(?P<callcont>\p{XID_Start}\p{XID_Continue}*)))\s*$|",
        // <retlabel> ret [variant]
        r"(?P<ret>((?P<retlabel>\p{XID_Start}\p{XID_Continue}*):\s+",
        r"ret\s+(?P<variant>([1-9][0-9]*|0))))\s*$|",
        // <grouplabel> group <element>*
        r"(?P<group>((?P<grouplabel>\p{XID_Start}\p{XID_Continue}*):\s+",
        r"group\s+(?P<elements>(\p{XID_Start}\p{XID_Continue}*\s*)*)))$|",
        // save <filename>
        r#"(?P<save>(save\s+(?P<savefilename>[^*?"<>|]+)))\s*$|"#,
        // load <filename>
        r#"(?P<load>(load\s+(?P<loadfilename>[^*?"<>|]+)))\s*$|"#,
        // test
        r#"(?P<test>test)\s*$|"#,
        // exit
        r"(?P<exit>exit)\s*$",
    ))
    .unwrap()
}
fn group_elements() -> Regex {
    Regex::new(r"\s+").unwrap()
}

fn captures_display(c: &Captures, names: &[&str]) {
    for name in names {
        if let Some(m) = c.name(name) {
            println!("{} = {}", name, m.as_str());
        }
    }
}

fn test(pm: &mut ProgramManager) -> Result<(), Error> {
    pm.define_call("fact", "fact1", 2, "fact19")?; //fact c n => fact1 c n fact_rec
    pm.define_call("fact1", "fact2", 3, "zero")?; //fact1 c n f => fact2 c n f zero

    pm.define_jmp("fact2", "fact3", "ba")?; //fact2 c n f z => fact3 n c f z
    pm.define_call("fact3", "copy_int", 1, "fact4")?; //fact3 n c f z => copy_int n (fact4 c f z)

    pm.define_jmp("fact4", "fact5", "aecdb")?; //fact4 n m c f z => fact5 n z c f m
    pm.define_call("fact5", "eq", 2, "fact6")?; //fact5 n z c f m => eq n z (fact6 c f m)

    pm.define_group("fact6", &["fact7", "fact11"])?; //fact6 => true: fact7, false: fact12

    pm.define_jmp("fact7", "fact8", "cba")?; //fact7 c f n => fact8 n f c
    pm.define_call("fact8", "drop_int", 1, "fact9")?; //fact8 n f c => drop n (fact9 f c)

    pm.define_call("fact9", "fact21", 1, "fact10")?; //fact9 f c => drop f (fact10 c)

    pm.define_call("fact10", "fact20", 1, "one")?; //fact10 c => fact11 c one

    pm.define_call("fact11", "fact12", 3, "one")?; //fact12 c f n => fact13 c f n one

    pm.define_jmp("fact12", "fact13", "cba")?; //fact13 c f n o => fact14 n f c o
    pm.define_call("fact13", "copy_int", 1, "fact14")?; //fact14 n f c o => copy_int n (fact15 f c o)

    pm.define_jmp("fact14", "fact15", "aecdb")?; //fact15 n m f c o => fact16 n o f c m
    pm.define_call("fact15", "minus", 2, "fact16")?; //fact16 n o c f m => minus n o (fact17 c f m)

    pm.define_jmp("fact16", "fact17", "ba")?; //fact17 n f c m => fact18 f n c m
    pm.define_call("fact17", "fact20", 2, "fact18")?; //fact18 n c m => fact19 f n (fact20 c m)

    pm.define_jmp("fact18", "mul", "acb")?; //fact20 n c m => mul m n c
    pm.define_group("fact19", &["fact", "fact20"])?; //fact_rec => call: fact, drop: fact_rec1

    pm.define_ret("fact20", 0)?; //fact11 c o => c o

    pm.define_ret("fact21", 1)?; //fact23 f c => f.1 c

    pm.set_export("fact")?;

    let mut file = File::create("fact.json")?;
    file.write_all(json!(pm).to_string().as_bytes())?;

    let prog = pm.compile()?;

    let mut ctx = Context::new();
    ctx.push(Value::FinalReceiver(print::<usize>));
    ctx.push(Value::wrap(10usize));
    prog.run(ctx, "fact", 0)?;

    Ok(())
}

fn process(c: Captures, names: &[&str], pm: &mut ProgramManager) -> Result<bool, Error> {
    captures_display(&c, names);
    if let Some(_) = c.name("test") {
        test(pm)?;
        return Ok(false);
    }
    if let Some(_) = c.name("save") {
        let filename = dbg!(c
            .name("savefilename")
            .expect("savefilename is none")
            .as_str()
            .trim());
        if let Ok(..) = std::fs::metadata(filename) {
            println!("{} is already exist. override?", filename);
            let mut buf=[0u8];
            std::io::stdin().read(&mut buf)?;
            if buf[0]!=b'Y' && buf[0]!=b'y' { return Ok(false) }
        }
        let mut file = File::create(filename)?;
        file.write_all(json!(pm).to_string().as_bytes())?;
        return Ok(false);
    }
    if let Some(_) = c.name("load") {
        let filename = dbg!(c
            .name("loadfilename")
            .expect("loadfilename is none")
            .as_str()
            .trim());
        let file = File::open(filename)?;
        let p: ProgramManager = serde_json::from_reader(file)?;
        dbg!(p.compile()?);
        return Ok(false);
    }
    if let Some(_) = c.name("exit") {
        dbg!(pm);
        return Ok(false);
    }
    if let Some(_) = c.name("jmp") {
        let jmplabel = c.name("jmplabel").expect("jmplabel is none").as_str();
        let jmpcont = c.name("jmpcont").expect("jmpcont is none").as_str();
        let per = c.name("per").expect("per is none").as_str();
        pm.define_jmp(jmplabel, jmpcont, &per[2..])
            .map(|e| info!("{:?}", e.access(&pm)))
            .unwrap_or_else(|e| error!("{}", e));
    }
    if let Some(_) = c.name("call") {
        let calllabel = c.name("calllabel").expect("jmplabel is none").as_str();
        let callee = c.name("callee").expect("callee is none").as_str();
        let callcnt = c
            .name("callcnt")
            .expect("callcnt is none")
            .as_str()
            .parse::<u8>()?;
        let callcont = c.name("callcont").expect("callcont is none").as_str();
        pm.define_call(calllabel, callee, callcnt, callcont)
            .map(|e| info!("{:?}", e.access(&pm)))
            .unwrap_or_else(|e| error!("{}", e));
    }
    if let Some(_) = c.name("ret") {
        let retlabel = c.name("retlabel").expect("retlabel is none").as_str();
        let variant = c
            .name("variant")
            .expect("callcnt is none")
            .as_str()
            .parse::<u8>()?;
        pm.define_ret(retlabel, variant)
            .map(|e| info!("{:?}", e.access(&pm)))
            .unwrap_or_else(|e| error!("{}", e));
    }
    if let Some(_) = c.name("group") {
        let grouplabel = c.name("grouplabel").expect("grouplabel is none").as_str();
        let elements = c.name("elements").expect("elements is none").as_str();
        let elements: Vec<&str> = group_elements().split(elements).collect();
        let elements: &[&str] = &elements;
        pm.define_group(grouplabel, elements)
            .map(|e| info!("{:?}", e.access(&pm)))
            .unwrap_or_else(|e| error!("{}", e));
    }

    Ok(true)
}

fn print_help() {
    println!(
        r#"Syntax (<label> are identifiers, [number:u8] are numbers): 
    <jmplabel>: jmp <jmpcont> #!<permutation>
    <calllabel>: call <callee> [callcnt:u8] <callcont>
    <retlabel>: ret [variant:u8]
    <grouplabel>: group <element>*
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
    let names: Vec<&str> = commands.capture_names().filter_map(|v| v).collect();

    print!("> ");
    stdout().flush()?;
    let sin = stdin();
    let mut pm = ProgramManager::new();
    let mut line;
    loop {
        line = "".into();
        sin.read_line(&mut line)?;
        if let Some(c) = commands.captures(&line) {
            if !(process(c, &names, &mut pm)?) {
                return Ok(());
            }
        } else {
            print_help();
        }
        print!("> ");
        stdout().flush()?;
    }
}
