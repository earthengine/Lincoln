use lincoln_compiled::value::Context;
use lincoln_compiled::value::Value;
use lincoln_compiled::coderef::CodeRef;
use lincoln_compiled::program::ExternEntry;

eval_fn_untyped!(_from(p, c), 2, [v, cont], {
    let v = v.unwrap::<usize>(p)?;
    debug!("from {}", v);
    let mut ctx: Context = Default::default();
    let from1 = CodeRef::ExternFn("from", _from);
    if v == 0 {
        cont.eval(p, c, 0)
    } else if v % 2 == 1 {
        let n = (v - 1) / 2;
        ctx.push(Value::wrap(n));
        c.push(Value::closure(&[from1], ctx));
        cont.eval(p, c, 1)
    } else {
        let n = (v - 2) / 2;
        ctx.push(Value::wrap(n));
        c.push(Value::closure(&[from1], ctx));
        cont.eval(p, c, 2)
    }
});
eval_fn_untyped!(_onzero(p, c), 1, [cont], {
    c.push(Value::wrap(0usize));
    cont.eval(p, c, 0)
});
eval_fn!(_onodd_result(p, c), 2, cont, [v]: [usize], {
    c.push(Value::wrap(v * 2 + 1));
    cont.eval(p, c, 0)
});
eval_fn_untyped!(_onodd(p, c), 2, [cont, v], {
    let mut ctx: Context = Default::default();
    ctx.push(cont);
    let closure = Value::closure(
        &[CodeRef::ExternFn(
            "onodd_result",
            _onodd_result,
        )],
        ctx,
    );
    c.push(v);
    c.push(closure);
    _count(p, c)
});
eval_fn!(_oneven_result(p, c), 2, cont, [v]: [usize], {
    c.push(Value::wrap(v * 2 + 2));
    cont.eval(p, c, 0)
});
eval_fn_untyped!(_oneven(p, c), 2, [cont, v], {
    let mut ctx: Context = Default::default();
    ctx.push(cont);
    let closure = Value::closure(
        &[CodeRef::ExternFn(
            "oneven_result",
            _oneven_result,
        )],
        ctx,
    );
    c.push(v);
    c.push(closure);
    _count(p, c)
});
eval_fn_untyped!(_count(p, c), 2, [cont, v], {
    debug!("count ");

    let mut ctx: lincoln_compiled::value::Context = Default::default();
    ctx.push(cont);
    let closure = Value::closure(
        &[
            lincoln_compiled::coderef::CodeRef::ExternFn("onzero", _onzero),
            lincoln_compiled::coderef::CodeRef::ExternFn("onodd", _onodd),
            lincoln_compiled::coderef::CodeRef::ExternFn("oneven", _oneven),
        ],
        ctx,
    );
    c.push(closure);
    v.eval(p, c, 0)
});

pub const BINT_EXTERNS: &[ExternEntry] = &[eval!("from", _from), eval!("count", _count)];
