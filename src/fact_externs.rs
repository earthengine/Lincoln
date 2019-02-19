use crate::coderef::CodeRef;
use crate::program::{ExternEntry, Program};
use crate::value::{Context, Value};
use core::any::Any;
use core::fmt::Display;
use failure::Error;

macro_rules! eval_fn_term {
    ($name:ident, $expect:expr, $t:ty, $v:ident, $blk:block) => {
pub fn $name<T>(p: &Program, mut c: Context) -> Result<(CodeRef, Context), Error>
where T: Display + Any + 'static
{
    c.expect_args($expect)?;
    let $v = c.pop()?.unwrap::<$t>(p)?;

    $blk
    Ok((CodeRef::Termination, c))
}
    };
}

eval_fn_term!(print, 1, usize, v, {
    println!("Result: {}", v);
});

macro_rules! eval_fn_binary {
    ($name:ident($p:ident, $c:ident), $expect:expr, $cont:ident, [$v1:ident: $t1: ty, $v2:ident: $t2: ty], $blk:block) => {
        pub fn $name($p: &Program, mut $c: Context) -> Result<(CodeRef, Context), Error> {
            $c.expect_args($expect)?;
            let $cont = $c.pop()?;
            let $v1 = $c.pop()?.unwrap::<$t1>($p)?;
            let $v2 = $c.pop()?.unwrap::<$t2>($p)?;

            $blk
        }
    };
}
macro_rules! eval_fn_unary {
    ($name:ident($p:ident, $c:ident), $expect:expr, $cont:ident, [$v1:ident: $t1: ty], $blk:block) => {
        pub fn $name($p: &Program, mut $c: Context) -> Result<(CodeRef, Context), Error> {
            $c.expect_args($expect)?;
            let $cont = $c.pop()?;
            let $v1 = $c.pop()?.unwrap::<$t1>($p)?;

            $blk
        }
    };
}

eval_fn_binary!(eq(p, c), 3, cont, [n1: usize, n2: usize], {
    println!("{}=={}: {}", n2, n1, n2 == n1);
    if n1 == n2 {
        cont.eval(p, c, 0)
    } else {
        cont.eval(p, c, 1)
    }
});
eval_fn_binary!(mul(p, c), 3, cont, [n1: usize, n2: usize], {
    println!("{}*{} = {}", n2, n1, n2 * n1);
    c.push(Value::wrap(n2 * n1));

    cont.eval(p, c, 0)
});
eval_fn_binary!(minus(p, c), 3, cont, [n1: usize, n2: usize], {
    println!("{}-{} = {}", n2, n1, n2 - n1);
    c.push(Value::wrap(n2 - n1));

    cont.eval(p, c, 0)
});

eval_fn_unary!(drop_int(p, c), 2, cont, [_v1: usize], {
    cont.eval(p, c, 0)
});
eval_fn_unary!(copy_int(p, c), 2, cont, [v: usize], {
    c.push(Value::wrap(v));
    c.push(Value::wrap(v));

    cont.eval(p, c, 0)
});

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
