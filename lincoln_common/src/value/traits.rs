use crate::{ValueAccessError, AnyDebugDisplay};
use crate::permutation::Permutation;
use core::fmt::Display;
use core::iter::once;

pub trait Acceptor<Value> {
    type Output;
    fn accept(&mut self, v: Value);
    fn finish(self) -> Self::Output;
}

pub trait Value: AnyDebugDisplay {
    fn take(&mut self) -> Box<dyn Value>;
}
pub trait Context: Display {
    fn empty_value(&self) -> Box<dyn Value>;
    fn create_empty(&self) -> Box<dyn Context>;
    fn permutate(&mut self, per: Permutation);
    fn take_after(&mut self, at: u8, values_accepter: &mut dyn FnMut(&mut dyn Value)) -> u8;
    fn extend(&'_ mut self, values: &mut dyn Iterator<Item = &mut dyn Value>);
    fn len(&self) -> u8;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A handy function for external functions. It checks that
/// there is exactly the amount of values being stored in the context.
///
/// args: the number of arguments expected.
///
pub trait ContextExt: Context {
    fn pop(&mut self) -> Result<Box<dyn Value>, ValueAccessError> {
        let mut values = [self.empty_value()];
        let c = self.take_after(self.len() - 1, &mut |value| {
            values[0] = value.take();
        });
        if c < 1 {
            Err(ValueAccessError::PopFromEmpty)
        } else {
            Ok(values[0].take())
        }
    }
    fn push(&mut self, mut v: Box<dyn Value>) {
        self.extend(&mut once(&mut *v));
    }

    fn split(&mut self, cnt: u8) -> Result<Box<dyn Context>, ValueAccessError> {
        if self.len() < cnt {
            return Err(ValueAccessError::SplitOutOfRange {
                at: 2,
                total: self.len(),
            });
        }
        let mut values = vec![];
        self.take_after(cnt, &mut |value| {
            values.push(value.take());
        });

        let mut result = self.create_empty();
        result.extend(&mut values.iter_mut().map(|x| &mut **x));
        Ok(result)
    }
    fn expect_args(&self, args: u8) -> Result<(), ValueAccessError> {
        if self.len() != args {
            Err(ValueAccessError::UnexpectedArgs {
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
    fn merge(&mut self, other: &mut dyn Context) {
        let mut values = vec![];
        other.take_after(0, &mut |value| {
            values.push(value.take());
        });
        self.extend(&mut values.iter_mut().map(|x| &mut **x));
    }
}

impl<T> ContextExt for T where T: Context + ?Sized {}
