use super::CodeRef;
use super::{Context, ContextExt, Value};
use crate::EvalError;
use core::fmt::Debug;
use core::fmt::Display;
use core::mem::replace;

pub(super) struct Closure {
    pub(super) tags: Vec<CodeRef>,
    pub(super) context: Box<dyn Context>,
}
impl Debug for Closure {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}
impl Display for Closure {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut it = self.tags.iter();
        write!(fmt, "[{{")?;
        if let Some(cr) = it.next() {
            write!(fmt, "{}", cr)?;
        }
        for cr in it {
            write!(fmt, ";{}", cr)?;
        }
        write!(fmt, "}} {}]", self.context)
    }
}
impl Value for Closure {
    fn eval(mut self: Box<Self>, ctx: &mut dyn Context, variant: u8) -> Result<CodeRef, EvalError> {
        let variant_cnt = self.tags.len();
        ctx.append(&mut *self.context);
        //A closure without variants is "Termination"
        if self.tags.is_empty() {
            return Ok(CodeRef::Termination);
        }
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
            let mut values = [
                Closure {
                    tags: self.tags.clone(),
                    context: ctx.create_empty(),
                },
                Closure {
                    tags: self.tags,
                    context: ctx.create_empty(),
                },
            ];
            ctx.put_many(&mut values.iter_mut().map(|c| -> &mut dyn Value { c }));
            cont.eval(ctx, 0)
        } else {
            Ok(self.tags[variant as usize])
        }
    }
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>> {
        None
    }
    fn take(&mut self) -> Box<dyn Value> {
        let empty = self.context.create_empty();
        let context = replace(&mut self.context, empty);
        let tags = replace(&mut self.tags, vec![]);
        Box::new(Closure { tags, context })
    }
}
