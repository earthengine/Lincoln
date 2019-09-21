use core::fmt::Display;
use core::fmt::Debug;
use super::{Value, Context, ContextExt};
use crate::coderef::CodeRef;
use crate::EvalError;
use smallvec::SmallVec;

pub(super) struct Closure(pub(super) SmallVec<[CodeRef; 5]>, pub(super) Box<dyn Context>);
impl Debug for Closure {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}
impl Display for Closure {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut it = self.0.iter();
        write!(fmt, "[{{")?;
        if let Some(cr) = it.next() {
            write!(fmt, "{}", cr)?;
        }
        for cr in it {
            write!(fmt, ";{}", cr)?;
        }
        write!(fmt, "}} {}]", self.1)
    }
}
impl Value for Closure {
    fn eval(mut self: Box<Self>, ctx: &mut dyn Context, variant: u8) -> Result<CodeRef, EvalError> {
        ctx.append(&mut *self.1);
        //A closure without variants is "Termination"
        if self.0.is_empty() {
            return Ok(CodeRef::Termination);
        }
        let variant_cnt = self.0.len();
        //Variant 1 is "drop" for single variant closures. Requires no captured variables
        //Variant 2 is "copy" for single variant closures. Requires no captured variables
        if variant as usize >= variant_cnt && (variant_cnt != 1 || (variant != 1 && variant != 2)) {
            Err(EvalError::VariantOutOfBound {
                given: variant,
                max: variant_cnt as u8,
            })
        } else if variant == 1 && variant_cnt == 1 {
            ctx.expect_args(1)?;
            let cont = ctx.pop()?;
            cont.eval(ctx, 0)
        } else if variant == 2 && variant_cnt == 1 {
            ctx.expect_args(1)?;
            let cont = ctx.pop()?;
            ctx.push(Box::new(Closure(self.0.clone(), ctx.create_empty())));
            ctx.push(Box::new(Closure(self.0, ctx.create_empty())));
            cont.eval(ctx, 0)
        } else {
            Ok(self.0[variant as usize])
        }
    }
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>> {
        None
    }
}