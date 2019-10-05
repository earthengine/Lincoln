/// This macro defines a wrapped value.
///
/// name: the name of the value
/// value: the value
///
macro_rules! value {
    ($name:expr, $exp:expr) => {
        || lincoln_compiled::ExternEntry::Value {
            name: $name.into(),
            value: lincoln_compiled::ValueFn::stateless(|| lincoln_compiled::wrap($exp)),
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
    ($ctx:ident, []:[]) => {
    };
    ($ctx:ident, [$var:ident]:[$typ:ty]) => {
        let $var = lincoln_compiled::unwrap::<$typ>(<_ as lincoln_compiled::ContextExt>::pop($ctx)?)?;
    };
    ($ctx:ident, [$var:ident,$($vars:ident),+]:[$typ:ty,$($typs:ty),+]) => {
        var_unwrap!($ctx, [$var]: [$typ]);
        var_unwrap!($ctx, [$($vars),*]:[$($typs),*])
    }
}

/// This macro take variables from the context, but do nothing with them.
///
/// ctx: the context
/// var(s): the name of the varialbes
///
macro_rules! var_pop {
    ($ctx:ident,[]) => {
    };
    ($ctx:ident,[$var:ident]) => {
        let $var = $ctx.pop()?;
    };
    ($ctx:ident,[$var:ident, $($vars:ident),+]) => {
        let $var = $ctx.pop()?;
        var_pop!($ctx, [$($vars),+])
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
    ($name:ident($ctx:ident), $varcnt:expr, $cont:ident, [$($var:ident),*]:[$($typ:ty),*], $blk:block) => {
        pub fn $name(
            $ctx: &mut dyn lincoln_compiled::Context,
        ) -> Result<lincoln_compiled::CodeRef,
            lincoln_compiled::EvalError,
        > {
            <_ as lincoln_compiled::ContextExt>::expect_args($ctx, $varcnt)?;
            let $cont = $ctx.pop()?;
            var_unwrap!($ctx, [$($var),*]:[$($typ),*]);

            $blk
        }
    };
}
/// This macro defines a function that can be used as an external function.
/// The types of the varialbes are unspecified, so they can be anything.
///
/// name: the name of the function
/// prog: the program parameter
/// ctx: the context parameter
/// varcnt: the expected number of variables
/// var(s): the name of the variables
/// blk: the function body with the variables defined as Value
///
macro_rules! eval_fn_untyped {
    ($name:ident($ctx:ident), $varcnt:expr, [$($var:ident),*], $blk:block) => {
        pub fn $name(
            $ctx: &mut dyn lincoln_compiled::Context,
        ) -> Result<lincoln_compiled::CodeRef, lincoln_compiled::EvalError> {
            <_ as lincoln_compiled::ContextExt>::expect_args($ctx, $varcnt)?;
            var_pop!($ctx,[$($var),*]);

            $blk
        }
    };
}

/// This macro defines a terminating function that can be used to create values.
/// Calling this function will always results in the termination of execution.
///
/// name: the name of the function
/// prog: the program parameter
/// ctx: the context parameter
/// var(s): the name of the variables
/// typ(s): the type of the variables
/// blk: the function body
///
macro_rules! eval_fn_term {
    ($name:ident($ctx:ident), [$($var:ident),*]:[$($typ:ty),*], $blk:block) => {
pub fn $name($ctx: &mut dyn lincoln_compiled::Context) ->
    Result<lincoln_compiled::CodeRef, lincoln_compiled::EvalError>
{
    var_unwrap!($ctx, [$($var),*]:[$($typ),*]);

    $blk
    Ok(lincoln_compiled::CodeRef::Termination)
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
            eval: lincoln_compiled::EvalFn::stateless($eval),
        }
    };
}

pub mod bint_externs;
pub mod fact_externs;

eval_fn_term!(print(c), []:[], {
    if c.is_empty() {
        println!("no result!");
    } else {
        let mut i=1;
        let len=c.len();
        while !c.is_empty() {
            var_unwrap!(c, [v]:[usize]);
            println!("Result({}/{}): {}", i,len,v);
            i+=1;
        }
    }
});
