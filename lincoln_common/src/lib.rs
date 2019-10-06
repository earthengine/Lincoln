#![deny(bare_trait_objects)]

mod traits;
pub use traits::{Access, AccessMut, AnyDebugDisplay, StringLike};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
