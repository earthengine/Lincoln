use core::any::Any;
use core::fmt::Display;
use crate::coderef::CodeRef;
use crate::program::{ExternEntry, Program};
use crate::value::{Context, Value};
use failure::Error;

pub fn print<T>(p: &Program, mut c: Context) -> Result<(CodeRef, Context), Error>
where T: Display + Any + 'static
{
    c.expect_args(1)?;
    let v = c.pop()?.unwrap::<T>(p)?;

    println!("Result: {}", v);
    Ok((CodeRef::Termination, c))
}

fn eq(p: &Program, mut c: Context) -> Result<(CodeRef, Context), Error> {
    c.expect_args(3)?;
    let cont = c.pop()?;
    let n1 = c.pop()?.unwrap::<usize>(p)?;
    let n2 = c.pop()?.unwrap::<usize>(p)?;

    if n1==n2 { cont.eval(p, c, 0) } else { cont.eval(p, c, 1) }
}

fn drop_int(p: &Program, mut c: Context) -> Result<(CodeRef, Context), Error> {
    c.expect_args(2)?;
    let cont = c.pop()?;
    let _v1 = c.pop()?.unwrap::<usize>(p)?;

    cont.eval(p, c, 0)
}

fn mul(p: &Program, mut c: Context) -> Result<(CodeRef, Context), Error> {
    c.expect_args(3)?;
    let cont = c.pop()?;
    let v1 = c.pop()?.unwrap::<usize>(p)?;
    let v2 = c.pop()?.unwrap::<usize>(p)?;

    println!("{}*{} = {}", v2, v1, v1 * v2);
    c.push(Value::wrap(v1*v2));

    cont.eval(p, c, 0)
}
fn minus(p: &Program, mut c: Context) -> Result<(CodeRef, Context), Error> {
    c.expect_args(3)?;
    let cont = c.pop()?;
    let v1 = c.pop()?.unwrap::<usize>(p)?;
    let v2 = c.pop()?.unwrap::<usize>(p)?;

    println!("{}-{}", v2, v1);
    c.push(Value::wrap(v2-v1));

    cont.eval(p, c, 0)
}
fn copy_int(p: &Program, mut c: Context) -> Result<(CodeRef, Context), Error> {
    c.expect_args(2)?;
    let cont = c.pop()?;
    let v = c.pop()?.unwrap::<usize>(p)?;
    c.push(Value::wrap(v));
    c.push(Value::wrap(v));
    cont.eval(p, c, 0)
}
pub const FACT_EXTERNS: &[ExternEntry] = &[
    ExternEntry::Value {
        name: "zero",
        value: || Value::wrap(0usize),
    },
    ExternEntry::Eval {
        name: "eq",
        eval: eq,
    },
    ExternEntry::Eval {
        name: "drop_int",
        eval: drop_int,
    },
    ExternEntry::Eval {
        name: "copy_int",
        eval: copy_int,
    },
    ExternEntry::Value {
        name: "one",
        value: || Value::wrap(1usize),
    },
    ExternEntry::Eval {
        name: "minus",
        eval: minus,
    },
    ExternEntry::Eval {
        name: "mul",
        eval: mul,
    },
];
