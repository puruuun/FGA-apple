#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareBy {
    Width(i32),
    Height(i32),
    None,
}
