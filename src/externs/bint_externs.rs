use crate::program::ExternEntry;
use crate::value::Value;

pub fn _from(
    p: &crate::program::Program,
    mut c: crate::value::Context,
) -> Result<(crate::coderef::CodeRef, crate::value::Context), failure::Error> {
    c.expect_args(2)?;
    let v = c.pop()?.unwrap::<usize>(p)?;
    let cont = c.pop()?;

    println!("v={}", v);
    let from1 = crate::coderef::CodeRef::ExternFn("from", _from);
    if v == 0 {
        cont.eval(p, c, 0)
    } else if v % 2 == 1 {
        let n = (v - 1) / 2;
        let mut ctx = crate::value::Context::new();
        ctx.push(Value::wrap(n));
        let closure = Value::closure(&[from1], ctx);
        c.push(closure);
        cont.eval(p, c, 1)
    } else {
        let n = (v - 2) / 2;
        let mut ctx = crate::value::Context::new();
        ctx.push(Value::wrap(n));
        let closure = Value::closure(&[from1], ctx);
        c.push(closure);
        cont.eval(p, c, 2)
    }
}

pub fn _count(
    p: &crate::program::Program,
    mut c: crate::value::Context,
) -> Result<(crate::coderef::CodeRef, crate::value::Context), failure::Error> {
    c.expect_args(2)?;
    let v1 = c.pop()?;
    let cont = c.pop()?;
    println!("count {:?} {:?}", cont, v1);

    let mut ctx = crate::value::Context::new();
    ctx.push(cont);
    let closure = Value::closure(
        &[
            crate::coderef::CodeRef::ExternFn("onzero", |p, mut c| {
                c.expect_args(1)?;
                let cont = c.pop()?;
                c.push(Value::wrap(0usize));
                cont.eval(p, c, 0)
            }),
            crate::coderef::CodeRef::ExternFn("onodd", |p, mut c| {
                c.expect_args(2)?;
                let cont = c.pop()?;
                let v = c.pop()?;
                let mut ctx = crate::value::Context::new();
                ctx.push(cont);
                let closure = Value::closure(
                    &[crate::coderef::CodeRef::ExternFn(
                        "onodd_result",
                        |p, mut c| {
                            c.expect_args(2)?;
                            let cont = c.pop()?;
                            let v = c.pop()?.unwrap::<usize>(p)?;
                            c.push(Value::wrap(v * 2 + 1));
                            cont.eval(p, c, 0)
                        },
                    )],
                    ctx,
                );
                c.push(closure);
                c.push(v);
                _count(p, c)
            }),
            crate::coderef::CodeRef::ExternFn("oneven", |p, mut c| {
                c.expect_args(2)?;
                let cont = c.pop()?;
                let v = c.pop()?;
                let mut ctx = crate::value::Context::new();
                ctx.push(cont);
                let closure = Value::closure(
                    &[crate::coderef::CodeRef::ExternFn(
                        "oneven_result",
                        |p, mut c| {
                            c.expect_args(2)?;
                            let cont = c.pop()?;
                            let v = c.pop()?.unwrap::<usize>(p)?;
                            c.push(Value::wrap(v * 2 + 2));
                            cont.eval(p, c, 0)
                        },
                    )],
                    ctx,
                );
                c.push(closure);
                c.push(v);
                _count(p, c)
            }),
        ],
        ctx,
    );
    c.push(closure);
    v1.eval(p, c, 0)
}

pub const BINT_EXTERNS: &[ExternEntry] = &[eval!("from", _from), eval!("count", _count)];
