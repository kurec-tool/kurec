pub enum ErrorAction {
    Retry,
    Ignore,
}
pub trait ClassifyError {
    fn error_action(&self) -> ErrorAction;
}
