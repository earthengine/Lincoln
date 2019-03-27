use crate::coderef::GroupRef;

#[derive(Fail, Debug)]
pub enum BuildError {
    #[fail(display = "Group {:?} not found", _0)]
    GroupNotFound(GroupRef),
    #[fail(display = "Given variant {} exceed limit {}", given, max)]
    VariangOutOfRange { max: u8, given: u8 },
}
