macro_rules! eval_fn_term {
    ($name:ident, $expect:expr, $t:ty, $v:ident, $blk:block) => {
pub fn $name<T>(p: &crate::program::Program, mut c: crate::value::Context) ->
    Result<(crate::coderef::CodeRef, crate::value::Context), failure::Error>
where T: std::fmt::Display + std::any::Any + 'static
{
    c.expect_args($expect)?;
    let $v = c.pop()?.unwrap::<$t>(p)?;

    $blk
    Ok((crate::coderef::CodeRef::Termination, c))
}
    };
}

macro_rules! value {
    ($name:expr, $v:expr) => {
        ExternEntry::Value {
            name: $name,
            value: || Value::wrap($v),
        }
    };
}

macro_rules! eval_fn_binary {
    ($name:ident($p:ident, $c:ident), $cont:ident, [$v1:ident: $t1: ty, $v2:ident: $t2: ty], $blk:block) => {
        pub fn $name(
            $p: &crate::program::Program,
            mut $c: crate::value::Context,
        ) -> Result<(crate::coderef::CodeRef, crate::value::Context), failure::Error> {
            $c.expect_args(3)?;
            let $cont = $c.pop()?;
            let $v1 = $c.pop()?.unwrap::<$t1>($p)?;
            let $v2 = $c.pop()?.unwrap::<$t2>($p)?;

            $blk
        }
    };
}
macro_rules! eval_fn_unary {
    ($name:ident($p:ident, $c:ident), $cont:ident, [$v1:ident: $t1: ty], $blk:block) => {
        pub fn $name(
            $p: &crate::program::Program,
            mut $c: crate::value::Context,
        ) -> Result<(crate::coderef::CodeRef, crate::value::Context), failure::Error> {
            $c.expect_args(2)?;
            let $cont = $c.pop()?;
            let $v1 = $c.pop()?.unwrap::<$t1>($p)?;

            $blk
        }
    };
}

macro_rules! eval {
    ($name:expr, $eval:expr) => {
        ExternEntry::Eval {
            name: $name,
            eval: $eval,
        }
    };
}

pub mod bint_externs;
pub mod fact_externs;

eval_fn_term!(print, 1, usize, v, {
    println!("Result: {}", v);
});
