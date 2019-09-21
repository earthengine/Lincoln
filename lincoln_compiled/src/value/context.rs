use crate::permutation::Permutation;
use super::{Value, Context};

/// A Context is a container of values.
/// Ideally it should not have more than 20 elements
/// but this is not a hard limit.
///
#[derive(Default)]
pub struct ContextImpl(pub(super) Vec<Box<dyn Value>>);
impl std::fmt::Display for ContextImpl {
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
impl Context for ContextImpl {
    fn create_empty(&self) -> Box<dyn Context> {
        Box::new(ContextImpl::default())
    }
    fn len(&self) -> u8 {
        return self.0.len() as u8
    }
    fn take_many(&mut self, values: &mut [Option<Box<dyn Value>>]) -> u8 {
        let mylen = self.len() as usize;
        let split_at = if mylen > values.len() { values.len() } else { mylen };
        let split_at = mylen - split_at;
        let r = self.0.split_off(split_at);
        for (i,v) in r.into_iter().enumerate(){
            values[i] = Some(v);
        }
        return split_at as u8;
    }
    fn put_many(&mut self, mut values: Vec<Box<dyn Value>>) {
        self.0.append(&mut values);
    }

    /// Perform a permutation over the values.
    ///
    /// p: the permutation to perform.
    ///
    fn permutate(&mut self, p: Permutation) {
        p.permutate(&mut self.0)
    }
}
impl Drop for ContextImpl {
    fn drop(&mut self) {}
}
