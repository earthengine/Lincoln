use lincoln_compiled::ExternEntry;

eval_fn!(_eq(c), 3, cont, [n1, n2]:[usize,usize], {
    debug!("{}=={}: {}", n2, n1, n2 == n1);
    if n1 == n2 {
        cont.eval(c, 0)
    } else {
        cont.eval(c, 1)
    }
});
eval_fn!(_mul(c), 3, cont, [n1,n2]:[usize,usize], {
    debug!("{}*{} = {}", n2, n1, n2 * n1);
    c.push(lincoln_compiled::wrap(n2 * n1));

    cont.eval(c, 0)
});
eval_fn!(_minus(c), 3, cont, [n1,n2]:[usize,usize], {
    debug!("{}-{} = {}", n2, n1, n2 - n1);
    c.push(lincoln_compiled::wrap(n2 - n1));

    cont.eval(c, 0)
});
eval_fn!(_try_minus(c), 3, cont, [n1,n2]:[usize,usize], {
    debug!("{}-{}", n2, n1);
    if n2>=n1 {
        debug!("{}-{} = {}", n2, n1, n2 - n1);
        c.push(lincoln_compiled::wrap(n2-n1));
        cont.eval(c, 0)
    } else {
        c.push(lincoln_compiled::wrap(n2));
        c.push(lincoln_compiled::wrap(n1));
        cont.eval(c, 1)
    }
});

eval_fn!(_drop_int(c), 2, cont, [_v1]: [usize], {
    cont.eval(c, 0)
});
eval_fn!(_copy_int(c), 2, cont, [v]: [usize], {
    c.push(lincoln_compiled::wrap(v));
    c.push(lincoln_compiled::wrap(v));

    cont.eval(c, 0)
});

pub const FACT_EXTERNS: &[fn() -> ExternEntry] = &[
    value!("zero", 0usize),
    eval!("eq", _eq),
    eval!("drop_int", _drop_int),
    eval!("copy_int", _copy_int),
    value!("one", 1usize),
    eval!("minus", _minus),
    eval!("try_minus", _try_minus),
    eval!("mul", _mul),
];
