use core::any::Any;
use core::fmt::Debug;

/// A trait that makes use of string like types (`String` and `&str`)
/// easier.
///
pub trait StringLike: Into<String> + AsRef<str> {
    /// Calling `into` will consume the object,
    /// calling `as_ref` only gives borrowed string.
    /// This combines them together to give a owned string
    /// without consuming the value.
    ///
    fn clone_string(&self) -> String {
        self.as_ref().to_string()
    }
}
impl<T> StringLike for T where T: Into<String> + AsRef<str> {}

/// A trait for field access
///
pub trait Access<'a, Source> {
    type Target: 'a;
    fn access<'b>(&self, src: &'b Source) -> Self::Target
    where
        'b: 'a;
}
pub trait AccessMut<'a, Source> {
    type Target: 'a;
    fn access_mut<'b>(&self, src: &'b mut Source) -> Self::Target
    where
        'b: 'a;
}

/// A wrapper trait for wrapped values.
/// It represents any types that are both `Any` and `Debug`.
///
pub trait AnyDebug: Any + Debug {
    /// Obtain a trait object for `Any`
    fn as_any(&self) -> &dyn Any;
    /// Obtain a trait object for `Debug`
    fn as_debug(&self) -> &dyn Debug;
    /// Turn a concret box into a box of `Any`
    fn into_boxed_any(self: Box<Self>) -> Box<dyn Any>;
}
impl<T> AnyDebug for T
where
    T: Any + Debug,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_debug(&self) -> &dyn Debug {
        self
    }
    fn into_boxed_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}