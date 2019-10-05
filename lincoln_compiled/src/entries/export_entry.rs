use crate::GroupRef;

#[derive(Clone, Serialize)]
pub struct ExportEntry {
    pub name: String,
    pub g: GroupRef,
}
impl std::fmt::Display for ExportEntry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}: {}", self.name, self.g)
    }
}
impl std::fmt::Debug for ExportEntry {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}
