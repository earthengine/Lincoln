use crate::coderef::{CodeRef, GroupRef};
use crate::entries::ExternEntry;
use crate::permutation::Permutation;
use crate::program::Program;
use crate::{CodeRefError, EvalError, ValueAccessError};
use core::fmt::Debug;
use core::fmt::Formatter;
use lincoln_common::traits::{Access, AnyDebug};
use smallvec::SmallVec;

pub trait Value: AnyDebug {
    fn eval(
        self: Box<Self>,
        p: &Program,
        ctx: Context,
        variant: u8,
    ) -> Result<(CodeRef, Context), EvalError>;
    fn into_wrapped(self: Box<Self>, prog: &Program) -> Result<Box<dyn Value>, EvalError>;
}
struct Closure(SmallVec<[CodeRef; 5]>, Context);
impl Debug for Closure {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{{{:?} {:?}}}", self.0, self.1)
    }
}
impl Value for Closure {
    fn eval(
        mut self: Box<Self>,
        _: &Program,
        mut ctx: Context,
        variant: u8,
    ) -> Result<(CodeRef, Context), EvalError> {
        ctx.append(&mut self.1);
        if variant as usize >= self.0.len() {
            Err(EvalError::VariantOutOfBound {
                given: variant,
                max: self.0.len() as u8,
            })
        } else {
            Ok((self.0[variant as usize].clone(), ctx))
        }
    }
    fn into_wrapped(self: Box<Self>, prog: &Program) -> Result<Box<dyn Value>, EvalError> {
        if self.1.len() > 0 {
            return Err(ValueAccessError::UnwrappingNonEmptyClosure.into());
        }
        if self.0.len() != 1 {
            return Err(ValueAccessError::UnwrappingMultivariantClosure.into());
        }
        match self.0[0] {
            CodeRef::Extern(ext) => {
                if let Some(ext) = ext.access(prog) {
                    match ext {
                        ExternEntry::Value { value, .. } => Ok(value.get_value()),
                        _ => Err(ValueAccessError::ExternNotValue.into()),
                    }
                } else {
                    Err(CodeRefError::ExternNotFound { index: ext }.into())
                }
            }
            _ => Err(CodeRefError::CodeRefNotExtern.into()),
        }
    }
}

/// A Context is a container of values.
/// Ideally it should not have more than 20 elements
/// but this is not a hard limit.
///
#[derive(Default)]
pub struct Context(Vec<Box<dyn Value>>);
impl std::fmt::Debug for Context {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn debug_fmt(b: &Box<dyn Value>, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(fmt, "{:?}", b)
        }

        write!(fmt, "(")?;
        let mut it = self.0.iter();
        if let Some(value) = it.next() {
            debug_fmt(value, fmt)?;
        }
        for value in it {
            debug_fmt(value, fmt)?;
        }
        write!(fmt, ")")
    }
}
impl Context {
    /// A handy function for external functions. It checks that
    /// there is exactly the amount of values being stored in the context.
    ///
    /// args: the number of arguments expected.
    ///
    pub fn expect_args(&self, args: u8) -> Result<(), EvalError> {
        if self.len() != args {
            return Err(EvalError::UnexpectedArgs {
                expect: args,
                actual: self.len(),
            });
        } else {
            Ok(())
        }
    }
    /// Merge two context into one. The second context put last.
    ///
    /// other: the other context to merge
    ///
    pub fn append(self: &mut Self, other: &mut Self) {
        self.0.append(&mut other.0);
    }
    /// Take all value out and append to a vector.
    pub fn iterate(self: &mut Self) -> impl Iterator<Item = Box<dyn Value>> {
        core::mem::replace(&mut self.0, vec![]).into_iter()
    }
    /// Receive the length of the context.
    ///
    pub fn len(&self) -> u8 {
        self.0.len() as u8
    }
    /// Split the context into two pieces. Used to construct
    /// closures.
    ///
    /// at: where to split up
    ///
    pub fn split(mut self, at: u8) -> Result<(Self, Self), ValueAccessError> {
        if at as usize > self.0.len() {
            return Err(ValueAccessError::SplitOutOfRange {
                at,
                total: self.len(),
            });
        }
        let r = self.0.split_off(at as usize);
        let s0 = std::mem::replace(&mut self.0, vec![]);
        let ctx1 = Context(s0);
        let ctx2 = Context(r);
        Ok((ctx1, ctx2))
    }
    /// Store one more value to the context. The lenghth increases
    /// by 1.
    ///
    /// v: the value to store
    ///
    pub fn push(self: &mut Self, v: Box<dyn Value>) {
        self.0.push(v)
    }
    /// Retrive a value from the context. The length decreases by 1.
    /// Fail if the context is empty.
    ///
    pub fn pop(&mut self) -> Result<Box<dyn Value>, ValueAccessError> {
        self.0.pop().ok_or(ValueAccessError::PopFromEmpty)
    }
    /// Perform a permutation over the values.
    ///
    /// p: the permutation to perform.
    ///
    pub fn permutate(&mut self, p: Permutation) {
        p.permutate(&mut self.0)
    }
}
impl Drop for Context {
    fn drop(&mut self) {}
}
pub(crate) fn closure_prog(
    ent: GroupRef,
    ctx: Context,
    prog: &Program,
) -> Result<Box<dyn Value>, EvalError> {
    Ok(Box::new(Closure(ent.get_vec(prog)?, ctx)))
}

struct Wrapped<T>(T);
impl<T> Debug for Wrapped<T>
where
    T: Debug,
{
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "|{:?}|", self.0)
    }
}
impl<T> Value for Wrapped<T>
where
    T: AnyDebug,
{
    fn eval(
        self: Box<Self>,
        _: &Program,
        _: Context,
        _: u8,
    ) -> Result<(CodeRef, Context), EvalError> {
        Err(EvalError::CallingWrapped)
    }
    fn into_wrapped(self: Box<Self>, _: &Program) -> Result<Box<dyn Value>, EvalError> {
        Ok(self)
    }
}
pub fn wrap<T>(t: T) -> Box<dyn Value>
where
    T: AnyDebug,
{
    Box::new(Wrapped(t))
}
pub fn unwrap<T>(v: Box<dyn Value>, prog: &Program) -> Result<T, EvalError>
where
    T: AnyDebug,
{
    Ok(v.into_wrapped(prog)?
        .into_boxed_any()
        .downcast::<Wrapped<T>>()
        .map_err(|_| ValueAccessError::UnwrapNotWrapped)?
        .0)
}

struct WrappedFn<F>(String, F);
impl<F> Debug for WrappedFn<F>
where
    F: FnOnce(&Program, Context, u8) -> Result<(CodeRef, Context), EvalError>,
{
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}
impl<F> Value for WrappedFn<F>
where
    F: FnOnce(&Program, Context, u8) -> Result<(CodeRef, Context), EvalError> + 'static,
{
    fn eval(
        self: Box<Self>,
        p: &Program,
        ctx: Context,
        variant: u8,
    ) -> Result<(CodeRef, Context), EvalError> {
        self.1(p, ctx, variant)
    }
    fn into_wrapped(self: Box<Self>, _: &Program) -> Result<Box<dyn Value>, EvalError> {
        Err(ValueAccessError::CannotTurnIntoWrapped.into())
    }
}
pub fn native_closure(
    name: impl Into<String>,
    f: impl FnOnce(&Program, Context, u8) -> Result<(CodeRef, Context), EvalError> + 'static,
) -> Box<dyn Value> {
    Box::new(WrappedFn(name.into(), f))
}
