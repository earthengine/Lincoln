use lincoln_compiled::program::ExternEntry;
use lincoln_compiled::value::Value;

eval_fn!(_eq(p, c), 3, cont, [n1, n2]:[usize,usize], {
    debug!("{}=={}: {}", n2, n1, n2 == n1);
    if n1 == n2 {
        cont.eval(p, c, 0)
    } else {
        cont.eval(p, c, 1)
    }
});
eval_fn!(_mul(p, c), 3, cont, [n1,n2]:[usize,usize], {
    debug!("{}*{} = {}", n2, n1, n2 * n1);
    c.push(Value::wrap(n2 * n1));

    cont.eval(p, c, 0)
});
eval_fn!(_minus(p, c), 3, cont, [n1,n2]:[usize,usize], {
    debug!("{}-{} = {}", n2, n1, n2 - n1);
    c.push(Value::wrap(n2 - n1));

    cont.eval(p, c, 0)
});

eval_fn!(_drop_int(p, c), 2, cont, [_v1]: [usize], {
    cont.eval(p, c, 0)
});
eval_fn!(_copy_int(p, c), 2, cont, [v]: [usize], {
    c.push(Value::wrap(v));
    c.push(Value::wrap(v));

    cont.eval(p, c, 0)
});

pub const FACT_EXTERNS: &[ExternEntry] = &[
    value!("zero", 0usize),
    eval!("eq", _eq),
    eval!("drop_int", _drop_int),
    eval!("copy_int", _copy_int),
    value!("one", 1usize),
    eval!("minus", _minus),
    eval!("mul", _mul),
];
