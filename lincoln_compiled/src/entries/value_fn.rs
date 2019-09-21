use crate::Value;

/// Represents an external function that can produce values.
/// It can be a function pointer or a
/// boxed closure.
pub enum ValueFn {
    Stateless(fn() -> Box<dyn Value>),
    Dyn(Box<dyn Fn() -> Box<dyn Value>>),
}
impl ValueFn {
    /// Call the internal function to produce a value
    pub fn get_value(&self) -> Box<dyn Value> {
        match self {
            ValueFn::Stateless(f) => f(),
            ValueFn::Dyn(bf) => bf(),
        }
    }
    /// Create from a stateless closure or function
    pub fn stateless(f: fn() -> Box<dyn Value>) -> Self {
        ValueFn::Stateless(f)
    }
    /// Create from a stateful closure (will be boxed)
    pub fn dynamic(f: impl 'static + Fn() -> Box<dyn Value>) -> Self {
        ValueFn::Dyn(Box::new(f))
    }
}
