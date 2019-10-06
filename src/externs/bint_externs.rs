use lincoln_common::ContextExt;
use lincoln_compiled::{EvalError, ExternEntry};

eval_fn_untyped!(_from(c), 2, [v, cont], {
    let v = lincoln_common::unwrap::<usize>(v)?;
    debug!("from {}", v);
    if v == 0 {
        lincoln_compiled::eval_closure(cont, c, 0)
    } else if v % 2 == 1 {
        let n = (v - 1) / 2;
        c.push(lincoln_compiled::native_closure("from", move |c, _| {
            c.push(lincoln_common::wrap(n));
            _from(c)
        }));
        lincoln_compiled::eval_closure(cont, c, 1)
    } else {
        let n = (v - 2) / 2;
        c.push(lincoln_compiled::native_closure("from", move |c, _| {
            c.push(lincoln_common::wrap(n));
            _from(c)
        }));
        lincoln_compiled::eval_closure(cont, c, 2)
    }
});
eval_fn_untyped!(_onzero(c), 1, [cont], {
    c.push(lincoln_common::wrap(0usize));
    lincoln_compiled::eval_closure(cont, c, 0)
});
eval_fn!(_onodd_result(c), 2, cont, [v]: [usize], {
    c.push(lincoln_common::wrap(v * 2 + 1));
    lincoln_compiled::eval_closure(cont, c, 0)
});
eval_fn_untyped!(_onodd(c), 2, [cont, v], {
    c.push(v);
    c.push(lincoln_compiled::native_closure("onodd", |c, _| {
        c.push(cont);
        _onodd_result(c)
    }));
    _count(c)
});
eval_fn!(_oneven_result(c), 2, cont, [v]: [usize], {
    c.push(lincoln_common::wrap(v * 2 + 2));
    lincoln_compiled::eval_closure(cont, c, 0)
});
eval_fn_untyped!(_oneven(c), 2, [cont, v], {
    c.push(v);
    c.push(lincoln_compiled::native_closure("oneven", |c, _| {
        c.push(cont);
        _oneven_result(c)
    }));
    _count(c)
});
eval_fn_untyped!(_count(c), 2, [cont, v], {
    debug!("count ");

    c.push(lincoln_compiled::native_closure("count_handler", |c, v| {
        c.push(cont);
        match v {
            0 => _onzero(c),
            1 => _onodd(c),
            2 => _oneven(c),
            n => Err(EvalError::VariantOutOfBound { given: n, max: 2 }),
        }
    }));
    lincoln_compiled::eval_closure(v, c, 0)
});

pub const BINT_EXTERNS: &[fn() -> ExternEntry] = &[eval!("from", _from), eval!("count", _count)];
