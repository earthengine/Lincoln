use lincoln_compiled::{EvalError, ExternEntry};

eval_fn_untyped!(_from(p, c), 2, [v, cont], {
    let v = lincoln_compiled::unwrap::<usize>(v, p)?;
    debug!("from {}", v);
    if v == 0 {
        cont.eval(p, c, 0)
    } else if v % 2 == 1 {
        let n = (v - 1) / 2;
        c.push(lincoln_compiled::native_closure(
            "from",
            move |p, mut c, _| {
                c.push(lincoln_compiled::wrap(n));
                _from(p, c)
            },
        ));
        cont.eval(p, c, 1)
    } else {
        let n = (v - 2) / 2;
        c.push(lincoln_compiled::native_closure(
            "from",
            move |p, mut c, _| {
                c.push(lincoln_compiled::wrap(n));
                _from(p, c)
            },
        ));
        cont.eval(p, c, 2)
    }
});
eval_fn_untyped!(_onzero(p, c), 1, [cont], {
    c.push(lincoln_compiled::wrap(0usize));
    cont.eval(p, c, 0)
});
eval_fn!(_onodd_result(p, c), 2, cont, [v]: [usize], {
    c.push(lincoln_compiled::wrap(v * 2 + 1));
    cont.eval(p, c, 0)
});
eval_fn_untyped!(_onodd(p, c), 2, [cont, v], {
    c.push(v);
    c.push(lincoln_compiled::native_closure("onodd", |p, mut c, _| {
        c.push(cont);
        _onodd_result(p, c)
    }));
    _count(p, c)
});
eval_fn!(_oneven_result(p, c), 2, cont, [v]: [usize], {
    c.push(lincoln_compiled::wrap(v * 2 + 2));
    cont.eval(p, c, 0)
});
eval_fn_untyped!(_oneven(p, c), 2, [cont, v], {
    c.push(v);
    c.push(lincoln_compiled::native_closure("oneven", |p, mut c, _| {
        c.push(cont);
        _oneven_result(p, c)
    }));
    _count(p, c)
});
eval_fn_untyped!(_count(p, c), 2, [cont, v], {
    debug!("count ");

    c.push(lincoln_compiled::native_closure(
        "cont_handler",
        |p, mut c, v| {
            c.push(cont);
            match v {
                0 => _onzero(p, c),
                1 => _onodd(p, c),
                2 => _oneven(p, c),
                n => return Err(EvalError::VariantOutOfBound { given: n, max: 2 }),
            }
        },
    ));
    v.eval(p, c, 0)
});

pub const BINT_EXTERNS: &[fn() -> ExternEntry] = &[eval!("from", _from), eval!("count", _count)];
