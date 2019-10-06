use crate::program::Program;
use crate::references::GroupRef;
use crate::entries::ExternEntry;
use super::CodeRef;
use lincoln_common::{Context, ContextExt, Value};
use crate::EvalError;
use core::fmt::{Debug, Display};
use core::mem::replace;
use lincoln_common::Access;

struct Closure {
    tags: Vec<CodeRef>,
    context: Box<dyn Context>,
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
    fn take(&mut self) -> Box<dyn Value> {
        let empty = self.context.create_empty();
        let context = replace(&mut self.context, empty);
        let tags = replace(&mut self.tags, vec![]);
        Box::new(Closure { tags, context })
    }
}
impl Closure {
    pub fn get_from_value(value: Box<dyn Value>) -> Result<Self, EvalError> {
        value.into_boxed_any()
            .downcast::<Closure>().map_err(|_| EvalError::CallingWrapped)
            .map(|c| *c)
    }
    pub fn eval(mut self: Self, ctx: &mut dyn Context, variant: u8) -> Result<CodeRef, EvalError> {
        let variant_cnt = self.tags.len();
        ctx.merge(&mut *self.context);
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
            Self::get_from_value(cont)?.eval(ctx, 0)
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
            ctx.extend(&mut values.iter_mut().map(|c| -> &mut dyn Value { c }));            
            Self::get_from_value(cont)?.eval(ctx, 0)
        } else {
            Ok(self.tags[variant as usize])
        }
    }
}

pub fn eval_closure(value: Box<dyn Value>, ctx: &mut dyn Context, variant: u8)
    -> Result<CodeRef, EvalError>
{
    Closure::get_from_value(value)?.eval(ctx, variant)
}

/// Build a closure value from a group reference, a context and program
///
pub(crate) fn closure_prog(
    ent: GroupRef,
    ctx: Box<dyn Context>,
    prog: &Program,
) -> Result<Box<dyn Value>, EvalError> {
    if let Some(1) = ent.count(prog) {
        if let Ok(CodeRef::Extern(ext)) = ent.get_entry(prog, 0) {
            if let Some(ExternEntry::Value { value, .. }) = ext.access(prog) {
                ctx.expect_args(0)?;
                return Ok(value.get_value());
            }
        }
    }

    Ok(Box::new(Closure {
        tags: ent.get_vec(prog)?.into_vec(),
        context: ctx,
    }))
}
