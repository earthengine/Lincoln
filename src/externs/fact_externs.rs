use crate::program::ExternEntry;
use crate::value::Value;

eval_fn_binary!(eq(p, c), cont, [n1: usize, n2: usize], {
    println!("{}=={}: {}", n2, n1, n2 == n1);
    if n1 == n2 {
        cont.eval(p, c, 0)
    } else {
        cont.eval(p, c, 1)
    }
});
eval_fn_binary!(mul(p, c), cont, [n1: usize, n2: usize], {
    println!("{}*{} = {}", n2, n1, n2 * n1);
    c.push(Value::wrap(n2 * n1));

    cont.eval(p, c, 0)
});
eval_fn_binary!(minus(p, c), cont, [n1: usize, n2: usize], {
    println!("{}-{} = {}", n2, n1, n2 - n1);
    c.push(Value::wrap(n2 - n1));

    cont.eval(p, c, 0)
});

eval_fn_unary!(drop_int(p, c), cont, [_v1: usize], { cont.eval(p, c, 0) });
eval_fn_unary!(copy_int(p, c), cont, [v: usize], {
    c.push(Value::wrap(v));
    c.push(Value::wrap(v));

    cont.eval(p, c, 0)
});

pub const FACT_EXTERNS: &[ExternEntry] = &[
    value!("zero", 0usize),
    eval!("eq", eq),
    eval!("drop_int", drop_int),
    eval!("copy_int", copy_int),
    value!("one", 1usize),
    eval!("minus", minus),
    eval!("mul", mul),
];
