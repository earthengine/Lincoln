use super::CodeRef;
use crate::error::EvalError;
use crate::Context;

/// Represents an external function that can be evaluated
/// It can be a function pointer or a
/// boxed closure.
pub enum EvalFn {
    Stateless(fn(&mut dyn Context) -> Result<CodeRef, EvalError>),
    Dyn(Box<dyn Fn(&mut dyn Context) -> Result<CodeRef, EvalError>>),
}
impl EvalFn {
    /// Call the internal function to evaluate the result
    pub fn eval(&self, ctx: &mut dyn Context) -> Result<CodeRef, EvalError> {
        match self {
            EvalFn::Stateless(f) => f(ctx),
            EvalFn::Dyn(bf) => bf(ctx),
        }
    }
    /// Create from a stateless closure or function
    pub fn stateless(f: fn(&mut dyn Context) -> Result<CodeRef, EvalError>) -> Self {
        EvalFn::Stateless(f)
    }
    /// Create from a stateful closure (will be boxed)
    pub fn stateful(bf: Box<dyn Fn(&mut dyn Context) -> Result<CodeRef, EvalError>>) -> Self {
        EvalFn::Dyn(bf)
    }
}
