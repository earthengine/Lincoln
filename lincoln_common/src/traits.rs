use core::any::Any;
use core::fmt::{Debug, Display};

/// A trait that makes use of string like types (`String` and `&str`)
/// easier.
///
pub trait StringLike {
    /// Calling `into` will consume the object,
    /// calling `as_ref` only gives borrowed string.
    /// This combines them together to give a owned string
    /// without consuming the value.
    ///
    fn to_string(self) -> String;
    fn as_str(&self) -> &str;
    fn clone_string(&self) -> String {
        self.as_str().into()
    }
}
impl<T> StringLike for T where T: Into<String> + AsRef<str> {
    fn to_string(self) -> String {
        self.into()
    }
    fn as_str(&self) -> &str {
        self.as_ref()
    }    
}

/// A trait for shared field access
///
pub trait Access<'a, Source> {
    /// The result type
    type Target: 'a;
    /// Given the source type, access its field
    fn access<'b>(&self, src: &'b Source) -> Self::Target
    where
        'b: 'a;
}
/// A trait for mutable field access
/// 
pub trait AccessMut<'a, Source> {
    /// The result type
    type Target: 'a;
    /// Given the source type, access its field
    fn access_mut<'b>(&self, src: &'b mut Source) -> Self::Target
    where
        'b: 'a;
}

/// A wrapper trait for wrapped values.
/// It represents any types that are both `Any`, `Debug` and `Display`.
///
pub trait AnyDebugDisplay: Any + Debug + Display {
    /// Obtain a trait object for `Any`
    fn as_any(&self) -> &dyn Any;
    /// Obtain a trait object for `Debug`
    fn as_debug(&self) -> &dyn Debug;
    /// Obtain a trait object for `Display`
    fn as_display(&self) -> &dyn Display;
    /// Turn a concret box into a box of `Any`
    fn into_boxed_any(self: Box<Self>) -> Box<dyn Any>;
}
impl<T> AnyDebugDisplay for T
where
    T: Any + Debug + Display,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_debug(&self) -> &dyn Debug {
        self
    }
    fn as_display(&self) -> &dyn Display {
        self
    }
    fn into_boxed_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}