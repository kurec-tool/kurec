#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorAction {
    Retry,
    Ignore,
}

/// エラーを分類し、適切なアクションを決定するためのトレイト
pub trait ClassifyError: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static {
    fn error_action(&self) -> ErrorAction;
}
