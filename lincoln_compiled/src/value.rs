use crate::coderef::{CodeRef, GroupRef};
use crate::entries::ExternEntry;
use crate::permutation::Permutation;
use crate::program::Program;
use crate::{EvalError, ValueAccessError};
use core::fmt::{Debug, Display, Formatter};
use lincoln_common::traits::{Access, AnyDebugDisplay};
use smallvec::SmallVec;

pub trait Value: AnyDebugDisplay {
    fn eval(self: Box<Self>, ctx: &mut Context, variant: u8) -> Result<CodeRef, EvalError>;
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>>;
}
struct Closure(SmallVec<[CodeRef; 5]>, Context);
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
    fn eval(mut self: Box<Self>, ctx: &mut Context, variant: u8) -> Result<CodeRef, EvalError> {
        ctx.append(&mut self.1);
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
            ctx.push(Box::new(Closure(self.0.clone(), Context::default())));
            ctx.push(Box::new(Closure(self.0, Context::default())));
            cont.eval(ctx, 0)
        } else {
            Ok(self.0[variant as usize])
        }
    }
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>> {
        None
    }
}

/// A Context is a container of values.
/// Ideally it should not have more than 20 elements
/// but this is not a hard limit.
///
#[derive(Default)]
pub struct Context(Vec<Box<dyn Value>>);
impl std::fmt::Display for Context {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "(")?;
        let mut it = self.0.iter();
        if let Some(value) = it.next() {
            write!(fmt, "{:?}", value)?;
        }
        for value in it {
            write!(fmt, ",{:?}", value)?;
        }
        write!(fmt, ")")
    }
}
impl Context {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// A handy function for external functions. It checks that
    /// there is exactly the amount of values being stored in the context.
    ///
    /// args: the number of arguments expected.
    ///
    pub fn expect_args(&self, args: u8) -> Result<(), EvalError> {
        if self.len() != args {
            Err(EvalError::UnexpectedArgs {
                expect: args,
                actual: self.len(),
            })
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
    pub fn split(&mut self, at: u8) -> Result<Self, ValueAccessError> {
        if at as usize > self.0.len() {
            return Err(ValueAccessError::SplitOutOfRange {
                at,
                total: self.len(),
            });
        }
        let r = self.0.split_off(at as usize);
        let ctx2 = Context(r);
        Ok(ctx2)
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
    if let Some(1) = ent.count(prog) {
        if let Ok(CodeRef::Extern(ext)) = ent.get_entry(prog, 0) {
            if let Some(ExternEntry::Value { value, .. }) = ext.access(prog) {
                ctx.expect_args(0)?;
                return Ok(value.get_value());
            }
        }
    }

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
impl<T> Display for Wrapped<T>
where
    T: Display,
{
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "|{}|", self.0)
    }
}
impl<T> Value for Wrapped<T>
where
    T: AnyDebugDisplay,
{
    fn eval(self: Box<Self>, _: &mut Context, _: u8) -> Result<CodeRef, EvalError> {
        Err(EvalError::CallingWrapped)
    }
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>> {
        Some(self)
    }
}
pub fn wrap<T>(t: T) -> Box<dyn Value>
where
    T: AnyDebugDisplay,
{
    Box::new(Wrapped(t))
}
pub fn unwrap<T>(v: Box<dyn Value>) -> Result<T, EvalError>
where
    T: AnyDebugDisplay,
{
    let fail = EvalError::from(ValueAccessError::UnwrapNotWrapped(
        "fail into_wrapped".into(),
    ));
    Ok(v.into_wrapped()
        .ok_or(fail)?
        .into_boxed_any()
        .downcast::<Wrapped<T>>()
        .map_err(|_| ValueAccessError::UnwrapNotWrapped("not Wrapped type".into()))?
        .0)
}

struct WrappedFn<F>(String, F);
impl<F> Debug for WrappedFn<F>
where
    F: FnOnce(&mut Context, u8) -> Result<CodeRef, EvalError>,
{
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}
impl<F> Display for WrappedFn<F>
where
    F: FnOnce(&mut Context, u8) -> Result<CodeRef, EvalError>,
{
    fn fmt(&self, fmt: &mut Formatter) -> core::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}
impl<F> Value for WrappedFn<F>
where
    F: FnOnce(&mut Context, u8) -> Result<CodeRef, EvalError> + 'static,
{
    fn eval(self: Box<Self>, ctx: &mut Context, variant: u8) -> Result<CodeRef, EvalError> {
        self.1(ctx, variant)
    }
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>> {
        None
    }
}
pub fn native_closure(
    name: impl Into<String>,
    f: impl FnOnce(&mut Context, u8) -> Result<CodeRef, EvalError> + 'static,
) -> Box<dyn Value> {
    Box::new(WrappedFn(name.into(), f))
}
