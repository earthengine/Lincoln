use crate::{copy_int, drop_int, eq, minus, mul, one, zero};
use lincoln_compiled::ExternEntry;

eval_fn!(_eq(p, c), 3, cont, [n1, n2]:[u32,u32], {
    if eq(n1,n2) {
        cont.eval(p, c, 0)
    } else {
        cont.eval(p, c, 1)
    }
});
eval_fn!(_mul(p, c), 3, cont, [n1,n2]:[u32,u32], {
    c.push(lincoln_compiled::wrap(mul(n1, n2)));

    cont.eval(p, c, 0)
});
eval_fn!(_minus(p, c), 3, cont, [n1,n2]:[u32,u32], {
    c.push(lincoln_compiled::wrap(minus(n2, n1)));

    cont.eval(p, c, 0)
});

eval_fn!(_drop_int(p, c), 2, cont, [n]: [u32], {
    drop_int(n);
    cont.eval(p, c, 0)
});
eval_fn!(_copy_int(p, c), 2, cont, [v]: [u32], {
    let r = copy_int(v);
    c.push(lincoln_compiled::wrap(r[0]));
    c.push(lincoln_compiled::wrap(r[1]));

    cont.eval(p, c, 0)
});

pub const FACT_EXTERNS: &[fn () -> ExternEntry] = &[
    value!("zero", zero()),
    eval!("eq", _eq),
    eval!("drop_int", _drop_int),
    eval!("copy_int", _copy_int),
    value!("one", one()),
    eval!("minus", _minus),
    eval!("mul", _mul),
];
