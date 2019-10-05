#![deny(bare_trait_objects)]

#[allow(clippy::borrowed_box)]
pub fn mut_box_to_mut<'a, T>(v: &'a mut Box<T>) -> &'a mut T
where
    T: ?Sized,
{
    &mut **v
}

pub mod traits;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
