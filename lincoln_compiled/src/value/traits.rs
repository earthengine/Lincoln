use crate::permutation::Permutation;
use crate::error::ValueAccessError;
use core::fmt::Display;
use crate::error::EvalError;
use crate::coderef::CodeRef;
use lincoln_common::traits::AnyDebugDisplay;

pub trait Value: AnyDebugDisplay {
    fn eval(self: Box<Self>, ctx: &mut dyn Context, variant: u8) -> Result<CodeRef, EvalError>;
    fn into_wrapped(self: Box<Self>) -> Option<Box<dyn Value>>;
}
pub trait Context : Display {
    fn create_empty(&self) -> Box<dyn Context>;
    fn permutate(&mut self, per: Permutation);
    fn take_many(&mut self, values: &mut [Option<Box<dyn Value>>]) -> u8;
    fn put_many(&mut self, values: Vec<Box<dyn Value>>);
    fn len(&self) -> u8;
}

/// A handy function for external functions. It checks that
/// there is exactly the amount of values being stored in the context.
///
/// args: the number of arguments expected.
///
pub trait ContextExt : Context {
    fn pop(&mut self) -> Result<Box<dyn Value>, ValueAccessError> {
        let mut values = [None];
        self.take_many(&mut values);
        if values[0].is_none() {
            Err(ValueAccessError::PopFromEmpty)
        } else {
            Ok(values[0].take().unwrap())
        }
    }
    fn push(&mut self, v: Box<dyn Value>) {
        self.put_many(vec![v]);
    }

    fn split(&mut self, at: u8) -> Result<Box<dyn Context>, ValueAccessError> {
        if self.len() < at {
            return Err(ValueAccessError::SplitOutOfRange { at: 2, total: self.len() });
        }
        let taken = self.len() - at;
        let mut val:Vec<Option<Box<dyn Value>>> = Vec::with_capacity(taken as usize);
        val.resize_with(taken as usize, || None);
        self.take_many(&mut val);

        let mut result = self.create_empty();
        let val:Vec<Box<dyn Value>> = val.into_iter().map(|x:Option<Box<dyn Value>>| x.unwrap()).collect();
        result.put_many(val);
        Ok(result)        
    }
    fn expect_args(&self, args: u8) -> Result<(), EvalError> {
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
    fn append(&mut self, other: &mut dyn Context) {
        let mut values:Vec<Option<Box<dyn Value>>> = Vec::with_capacity(other.len() as usize);
        values.resize_with(other.len() as usize, || None);
        other.take_many(&mut values);
        let values:Vec<Box<dyn Value>> = values.into_iter().map(|x| x.unwrap()).collect();
        self.put_many(values);
    }
    fn is_empty(&self) -> bool {
        self.len()==0
    }
}

impl<T> ContextExt for T where T: Context + ?Sized {}
