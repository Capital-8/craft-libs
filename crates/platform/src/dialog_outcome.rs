//! Dialog results with evidence: an empty rfd answer does not establish user cancellation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Capability {
    Available,
    Unavailable(String),
    Unknown(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Unavailable,
    Worker,
    Backend,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogError {
    pub kind: ErrorKind,
    pub message: String,
}
impl DialogError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into() }
    }
}
impl std::fmt::Display for DialogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for DialogError {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome<T> {
    Selected(T),
    /// Explicit app/backend cancellation; never inferred from rfd's empty answer.
    Cancelled,
    Failed(DialogError),
    /// No selection; rfd cannot say whether the user cancelled or its backend failed.
    Indeterminate,
}
impl<T> Outcome<T> {
    pub fn from_rfd(value: Option<T>) -> Self {
        value.map_or(Self::Indeterminate, Self::Selected)
    }
    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> Outcome<U> {
        match self {
            Self::Selected(value) => Outcome::Selected(map(value)),
            Self::Cancelled => Outcome::Cancelled,
            Self::Failed(error) => Outcome::Failed(error),
            Self::Indeterminate => Outcome::Indeterminate,
        }
    }
    pub fn failed(message: impl Into<String>) -> Self {
        Self::Failed(DialogError::new(ErrorKind::Backend, message))
    }
}
impl<T> Outcome<Vec<T>> {
    pub fn from_rfd_files(value: Option<Vec<T>>) -> Self {
        Self::from_rfd(value.filter(|paths| !paths.is_empty()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_and_empty_rfd_answers_never_establish_cancellation() {
        assert_eq!(Outcome::<String>::from_rfd(None), Outcome::Indeterminate);
        assert_eq!(Outcome::<Vec<String>>::from_rfd_files(Some(Vec::new())), Outcome::Indeterminate);
        assert_eq!(Outcome::from_rfd_files(Some(vec!["a"])), Outcome::Selected(vec!["a"]));
        for _capability in [Capability::Available, Capability::Unavailable("missing".into()), Capability::Unknown("timeout".into())] {
            assert_eq!(Outcome::<String>::from_rfd(None), Outcome::Indeterminate);
        }
    }
    #[test]
    fn mapping_preserves_explicit_cancellation_and_failure() {
        assert_eq!(Outcome::<u8>::Cancelled.map(u32::from), Outcome::Cancelled);
        assert_eq!(Outcome::<u8>::failed("worker stopped").map(u32::from), Outcome::failed("worker stopped"));
        assert_eq!(Outcome::Selected(7_u8).map(u32::from), Outcome::Selected(7));
    }
}
