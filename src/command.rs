use crate::compiled::coderef::CodeRef;
use crate::compiled::program::Program;
use crate::compiled::value::{Context, Value};
use crate::externs::bint_externs::BINT_EXTERNS;
use crate::externs::fact_externs::FACT_EXTERNS;
use crate::externs::print;
use crate::pre_compile::PreCompileProgram;
use crate::traits::Access;
use core::fmt::{Display, Formatter};
use failure::Error;
use regex::{Captures, Regex};
use std::fs::File;
use std::io::Write;

pub fn commands() -> Regex {
    Regex::new(concat!(
        // <jmplabel> jmp <jmpcont> #!<permutation>
        r#"^\s*(?P<jmp>((?P<jmplabel>\p{XID_Start}\p{XID_Continue}*):\s+"#,
        r#"jmp\s+(?P<jmpcont>\p{XID_Start}\p{XID_Continue}*)\s*(?P<per>#![a-t]{0,20})))(\s*//.*)?\s*$|"#,
        // <calllabel> call <callee> [callcnt] <callcont>
        r#"^\s*(?P<call>((?P<calllabel>\p{XID_Start}\p{XID_Continue}*):\s+"#,
        r#"call\s+(?P<callee>\p{XID_Start}\p{XID_Continue}*)\s+(?P<callcnt>([1-9][0-9]*|0))\s+"#,
        r#"(?P<callcont>\p{XID_Start}\p{XID_Continue}*)))(\s*//.*)?\s*$|"#,
        // <retlabel> ret [variant]
        r#"^\s*(?P<ret>((?P<retlabel>\p{XID_Start}\p{XID_Continue}*):\s+"#,
        r#"ret\s+(?P<variant>([1-9][0-9]*|0))))(\s*//.*)?\s*$|"#,
        // <grouplabel> group <element>*
        r#"^\s*(?P<group>((?P<grouplabel>\p{XID_Start}\p{XID_Continue}*):\s+"#,
        r#"group\s+(?P<elements>(\p{XID_Start}\p{XID_Continue}*\s*)*)))(\s*//.*)?\s*$|"#,
        // setexport <label>
        r#"^\s*(?P<setexport>set\s+export\s+(?P<exportlabel>\p{XID_Start}\p{XID_Continue}*))(\s*//.*)?\s*$|"#,
        // delete <label>
        r#"^\s*(?P<delete>delete\s+(?P<deletelabel>\p{XID_Start}\p{XID_Continue}*))\s*$|"#,
        // save <filename>
        r#"^\s*(?P<save>(save\s+(?P<savefilename>[^*?"<>|]+)))\s*$|"#,
        // load <filename>
        r#"^\s*(?P<load>(load\s+(?P<loadfilename>[^*?"<>|]+)))\s*$|"#,
        // show program
        r#"^\s*(?P<showprog>show\s+program)\s*$|"#,
        // show external set
        r#"^\s*(?P<showexternset>show\s+external\s+set)\s*$|"#,
        // compile <external set>
        r#"^\s*(?P<compile>compile\s+(?P<externalset_compile>\p{XID_Start}\p{XID_Continue}*))\s*$|"#,
        // run <external set> variant <value>
        // run <external set> variant <value> step
        r#"^\s*(?P<run>run\s+(?P<exportlabel_run>\p{XID_Start}\p{XID_Continue}*)\s+"#,
        r#"(?P<runvariant>([1-9][0-9]*|0))\s+"(?P<value>[^"]*)"(\s+(?P<runstep>step))?)\s*$|"#,
        // step
        r#"^\s*(?P<step>step)\s*$|"#,
        // empty line
        r#"^\s*(//.*)?\s*$|"#,
        // exit
        r#"^\s*(?P<exit>exit)\s*$"#,
    ))
    .unwrap()
}

pub enum CommandContext {
    Idle {
        program: PreCompileProgram,
        compiled: Option<Program>,
    },
    Stepping {
        program: PreCompileProgram,
        compiled: Program,
        context: Context,
        current: CodeRef,
        round: usize,
    },
}
impl Display for CommandContext {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        use CommandContext::*;
        match self {
            Idle { program, compiled } => write!(
                fmt,
                "program:\n{}\ncompiled:\n{}",
                program,
                compiled
                    .as_ref()
                    .map(|v| format!("{:?}", v))
                    .unwrap_or("".into())
            ),
            Stepping {
                program,
                compiled,
                context,
                current,
                round,
            } => {
                write!(fmt, "program:\n{}\ncompiled:\n{:?}", program, compiled)?;
                write!(
                    fmt,
                    "context:\n{:?}\ncurrent:\n{:?}\nround:{}",
                    context, current, round
                )
            }
        }
    }
}
impl Default for CommandContext {
    fn default() -> Self {
        CommandContext::Idle {
            program: Default::default(),
            compiled: Default::default(),
        }
    }
}
fn prompt_and_ask(prompt: impl AsRef<str>) -> Result<bool, Error> {
    println!("{}", prompt.as_ref());
    let mut line = "".into();
    let _ = std::io::stdin().read_line(&mut line)?;
    Ok(line.trim() == "Y" || line.trim() == "y")
}
impl CommandContext {
    fn program(&self) -> &PreCompileProgram {
        use CommandContext::*;
        match self {
            Idle { program, .. } | Stepping { program, .. } => program,
        }
    }
    fn program_mut(&mut self) -> &mut PreCompileProgram {
        use CommandContext::*;
        match self {
            Idle { program, .. } | Stepping { program, .. } => program,
        }
    }
    fn save(&mut self, c: Captures) -> Result<bool, Error> {
        let filename = c
            .name("savefilename")
            .expect("savefilename is none")
            .as_str()
            .trim();
        if let Ok(..) = std::fs::metadata(filename) {
            if !prompt_and_ask(format!("{} is already exist. override?", filename))? {
                return Ok(true);
            }
        }
        let mut file = File::create(filename)?;
        file.write_all(serde_json::to_string_pretty(&json!(self.program()))?.as_bytes())?;
        println!("saved to {}!", filename);
        Ok(true)
    }
    fn load(&mut self, c: Captures) -> Result<bool, Error> {
        let filename = c
            .name("loadfilename")
            .expect("loadfilename is none")
            .as_str()
            .trim();
        print!("loading {} ..", filename);
        let file = File::open(filename)?;
        let p: PreCompileProgram = serde_json::from_reader(file)?;
        self.program_mut().merge(&p)?;
        println!(" loaded.");
        Ok(true)
    }

    fn jmp(&mut self, c: Captures) -> Result<bool, Error> {
        let jmplabel = c.name("jmplabel").expect("jmplabel is none").as_str();
        let jmpcont = c.name("jmpcont").expect("jmpcont is none").as_str();
        let per = c.name("per").expect("per is none").as_str();
        let pm = self.program_mut();
        pm.define_jmp(jmplabel, jmpcont, &per[2..])
            .map(|e| info!("{:?}", e.access(&pm)))?;
        Ok(true)
    }
    fn call(&mut self, c: Captures) -> Result<bool, Error> {
        let calllabel = c.name("calllabel").expect("jmplabel is none").as_str();
        let callee = c.name("callee").expect("callee is none").as_str();
        let callcnt = c
            .name("callcnt")
            .expect("callcnt is none")
            .as_str()
            .parse::<u8>()?;
        let callcont = c.name("callcont").expect("callcont is none").as_str();
        let pm = self.program_mut();
        pm.define_call(calllabel, callee, callcnt, callcont)
            .map(|e| info!("{:?}", e.access(&pm)))?;
        Ok(true)
    }
    fn ret(&mut self, c: Captures) -> Result<bool, Error> {
        let retlabel = c.name("retlabel").expect("retlabel is none").as_str();
        let variant = c
            .name("variant")
            .expect("callcnt is none")
            .as_str()
            .parse::<u8>()?;
        let pm = self.program_mut();
        pm.define_ret(retlabel, variant)
            .map(|e| info!("{:?}", e.access(&pm)))?;
        Ok(true)
    }
    fn group(&mut self, c: Captures) -> Result<bool, Error> {
        let grouplabel = c.name("grouplabel").expect("grouplabel is none").as_str();
        let elements = c.name("elements").expect("elements is none").as_str();
        let elements: Vec<&str> = group_elements().split(elements).collect();
        let elements: &[&str] = &elements;
        let pm = self.program_mut();
        pm.define_group(grouplabel, elements)
            .map(|e| info!("{:?}", e.access(&pm)))?;
        Ok(true)
    }
    fn setexport(&mut self, c: Captures) -> Result<bool, Error> {
        let exportlabel = c.name("exportlabel").expect("exportlabel is none").as_str();
        let pm = self.program_mut();
        pm.set_export(exportlabel)?;
        Ok(true)
    }
    fn showprog(&mut self, _c: Captures) -> Result<bool, Error> {
        let pm = self.program();
        println!("{}", pm);
        Ok(true)
    }
    fn showexternset(&mut self, _: Captures) -> Result<bool, Error> {
        println!("fact");
        println!("bint");
        Ok(true)
    }
    fn delete(&mut self, c: Captures) -> Result<bool, Error> {
        let label = c.name("deletelabel").expect("deletelabel is none").as_str();
        self.program_mut().delete_ent(label);
        Ok(true)
    }
    fn compile(&mut self, c: Captures) -> Result<bool, Error> {
        let externs = c
            .name("externalset_compile")
            .expect("externalset_compile is none")
            .as_str();
        let externs = match externs {
            "fact" => FACT_EXTERNS,
            "bint" => BINT_EXTERNS,
            _ => bail!("extern set not found: {}", externs),
        };
        use CommandContext::*;
        match self {
            Idle {
                program,
                ref mut compiled,
            } => {
                *compiled = Some(program.compile(externs)?);
                Ok(true)
            }
            Stepping {
                ref mut program, ..
            } => {
                if !prompt_and_ask("You are in stepping mode. Quit?")? {
                    let program = std::mem::replace(program, Default::default());
                    let compiled = program.compile(externs)?;
                    *self = Idle {
                        program,
                        compiled: Some(compiled),
                    };
                }
                Ok(true)
            }
        }
    }

    fn run(&mut self, c: Captures) -> Result<bool, Error> {
        use CommandContext::*;
        let entry = c
            .name("exportlabel_run")
            .expect("exportlabel_run is none")
            .as_str();
        let variant = c
            .name("runvariant")
            .expect("runvariant is none")
            .as_str()
            .parse::<u8>()?;
        let values = c.name("value").expect("value is none").as_str();
        let values = Value::parse_string(values)?;
        let step = c.name("runstep").map(|_| true).unwrap_or(false);
        let mut ctx: Context = Default::default();
        ctx.push(Value::FinalReceiver(print));
        for value in values {
            ctx.push(value);
        }
        let (program, compiled, is_stepping) = match self {
            Idle { compiled: None, .. } => {
                bail!("Program is not compiled. Please compile it first (use compile command)")
            }
            Idle {
                compiled: Some(compiled),
                program,
            } => (program, compiled, false),
            Stepping {
                program, compiled, ..
            } => (program, compiled, true),
        };

        if is_stepping && !prompt_and_ask("You are stepping into the program. Restart?")? {
            return Ok(true);
        }

        if !step {
            compiled.run(ctx, entry, variant, None)?;
        } else {
            let entry = compiled.get_export_ent(entry, variant)?;
            println!("{:?} {:?}", entry, ctx);
            let program = std::mem::replace(program, Default::default());
            let compiled = std::mem::replace(compiled, Default::default());
            *self = Stepping {
                program,
                compiled,
                current: entry,
                context: ctx,
                round: 1,
            };
        };

        Ok(true)
    }
    fn step(&mut self, _c: Captures) -> Result<bool, Error> {
        use CommandContext::*;
        let (program, compiled, context, current, round) = match self {
            Stepping {
                compiled,
                context,
                current,
                round,
                program,
            } => (program, compiled, context, current, round),
            _ => bail!("Not in stepping mode. Run the program in step mode first."),
        };
        let ctx1 = std::mem::replace(context, Default::default());
        let (next, ctx1) = compiled.eval(ctx1, &current)?;
        *round += 1;
        if let CodeRef::Termination = next {
            *self = Idle {
                program: std::mem::replace(program, Default::default()),
                compiled: Some(std::mem::replace(compiled, Default::default())),
            }
        } else {
            println!("{}: {:?} {:?}", round, next, ctx1);
            *context = ctx1;
            *current = next;
        }
        Ok(true)
    }
}
macro_rules! handle_cmd {
    ($cmd: ident, $c:expr, $pm:expr) => {
        if let Some(_) = $c.name(stringify!($cmd)) {
            return $pm.$cmd($c);
        }
    };
}

pub fn process(c: Captures, ctx: &mut CommandContext) -> Result<bool, Error> {
    handle_cmd!(showprog, c, ctx);
    handle_cmd!(showexternset, c, ctx);
    handle_cmd!(save, c, ctx);
    handle_cmd!(load, c, ctx);
    handle_cmd!(jmp, c, ctx);
    handle_cmd!(call, c, ctx);
    handle_cmd!(ret, c, ctx);
    handle_cmd!(group, c, ctx);
    handle_cmd!(setexport, c, ctx);
    handle_cmd!(compile, c, ctx);
    handle_cmd!(run, c, ctx);
    handle_cmd!(step, c, ctx);
    handle_cmd!(delete, c, ctx);
    if let Some(_) = c.name("exit") {
        println!("{}", ctx);
        return Ok(false);
    }

    Ok(true)
}

fn group_elements() -> Regex {
    Regex::new(r"\s+").unwrap()
}
