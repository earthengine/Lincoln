/// This macro defines a wrapped value.
///
/// name: the name of the value
/// value: the value
///
macro_rules! value {
    ($name:expr, $exp:expr) => {
        || lincoln_compiled::ExternEntry::Value {
            name: $name.into(),
            value: Box::new(|| lincoln_compiled::wrap($exp)),
        }
    };
}

/// This macro takes one or more value(s) from the context, then unwrap it within the program.var_unwrap!
///
/// ctx: the context
/// prog: the program
/// var(s): the name of variables
/// typ(s): the type of variables
///
/// The number of variables and types must match.
macro_rules! var_unwrap {
    ($ctx:ident,$prog:ident, []:[]) => {
    };
    ($ctx:ident,$prog:ident, [$var:ident]:[$typ:ty]) => {
        let $var = lincoln_compiled::unwrap::<$typ>($ctx.pop()?, $prog)?;
    };
    ($ctx:ident,$prog:ident, [$var:ident,$($vars:ident),+]:[$typ:ty,$($typs:ty),+]) => {
        var_unwrap!($ctx, $prog, [$var]: [$typ]);
        var_unwrap!($ctx, $prog, [$($vars),*]:[$($typs),*])
    }
}

/// This macro defines a typed function that can be used as a external function
///
/// name: the name of the function
/// prog: the program parameter
/// ctx: the context parameter
/// varcnt: the expected number of variables in the context, including the continuation
/// cont: the "continuation" or returning variable, expected to be a closure so untyped
/// var(s): the name of the variables
/// typ(s): tye types of the variables
/// blk: the function body with the variables defined
///
macro_rules! eval_fn {
    ($name:ident($prog:ident, $ctx:ident), $varcnt:expr, $cont:ident, [$($var:ident),*]:[$($typ:ty),*], $blk:block) => {
        pub fn $name(
            $prog: &lincoln_compiled::Program,
            mut $ctx: lincoln_compiled::Context,
        ) -> Result<
            (
                lincoln_compiled::CodeRef,
                lincoln_compiled::Context,
            ),
            failure::Error,
        > {
            $ctx.expect_args($varcnt)?;
            let $cont = $ctx.pop()?;
            var_unwrap!($ctx,$prog, [$($var),*]:[$($typ),*]);

            $blk
        }
    };
}

/// This macro creates an ExternEntry with a given name and an evaluable function.
///
/// name: the name of the entry
/// eval: the function or closure
macro_rules! eval {
    ($name:expr, $eval:expr) => {
        || lincoln_compiled::ExternEntry::Eval {
            name: $name.into(),
            eval: Box::new($eval),
        }
    };
}

pub mod fact_externs;
pub use fact_externs::FACT_EXTERNS;