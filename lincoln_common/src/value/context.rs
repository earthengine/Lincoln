use core::fmt::{Display, Formatter};
use super::{Context, Value, Wrapped};
use crate::permutation::Permutation;

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
#[derive(Debug)]
enum Bottom {}
impl Display for Bottom {
    fn fmt(&self, _:&mut Formatter) -> std::fmt::Result { Ok(()) }
}
impl Context for ContextImpl {
    fn empty_value(&self) -> Box<dyn Value> {
        Box::new(<Wrapped<Bottom> as Default>::default())
    }
    fn create_empty(&self) -> Box<dyn Context> {
        Box::new(ContextImpl::default())
    }
    fn len(&self) -> u8 {
        self.0.len() as u8
    }
    /// Perform a permutation over the values.
    ///
    /// p: the permutation to perform.
    ///
    fn permutate(&mut self, p: Permutation) {
        p.permutate(&mut self.0)
    }
    fn take_after(&mut self, at: u8, values_accepter: &mut dyn FnMut(&mut dyn Value)) -> u8 {
        let at = if self.0.len() < at as usize {
            0
        } else {
            at as usize
        };
        let mut values = self.0.split_off(at);
        for value in values.iter_mut() {
            values_accepter(&mut **value);
        }
        values.len() as u8
    }
    fn extend(&mut self, values: &mut dyn Iterator<Item = &mut dyn Value>) {
        self.0.extend(values.map(|x| x.take()));
    }
}
impl Drop for ContextImpl {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod test {
    use crate::value::context::ContextImpl;
    use crate::value::traits::{Context, ContextExt};
    use crate::value::{unwrap, wrap};
    #[test]
    fn test_take_values() {
        let mut c = ContextImpl(vec![]);
        c.push(wrap(10i32));
        assert_eq!(1, c.take_after(0, &mut |v| ()));
    }

    #[test]
    fn test_pop_push() {
        let mut c = ContextImpl(vec![]);
        c.push(wrap(10i32));
        assert_eq!(1, c.len());
        assert_eq!(10i32, unwrap::<i32>(c.pop().unwrap()).unwrap());
        assert!(c.is_empty());
    }
}
