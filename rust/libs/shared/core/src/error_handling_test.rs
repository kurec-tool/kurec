use crate::error_handling::{ClassifyError, ErrorAction};
use std::fmt;

// テスト用のエラー型
#[derive(Debug)]
struct RetryError {
    pub message: String,
}

impl fmt::Display for RetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RetryError: {}", self.message)
    }
}

impl std::error::Error for RetryError {}

// テスト用のエラー型
#[derive(Debug)]
struct IgnoreError {
    pub message: String,
}

impl fmt::Display for IgnoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IgnoreError: {}", self.message)
    }
}

impl std::error::Error for IgnoreError {}

// テスト用のエラー型（動的に設定可能）
#[derive(Debug)]
struct DynamicError {
    pub message: String,
    pub should_retry: bool,
}

impl fmt::Display for DynamicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicError: {}", self.message)
    }
}

impl std::error::Error for DynamicError {}

impl ClassifyError for RetryError {
    fn error_action(&self) -> ErrorAction {
        ErrorAction::Retry
    }
}

impl ClassifyError for IgnoreError {
    fn error_action(&self) -> ErrorAction {
        ErrorAction::Ignore
    }
}

impl ClassifyError for DynamicError {
    fn error_action(&self) -> ErrorAction {
        if self.should_retry {
            ErrorAction::Retry
        } else {
            ErrorAction::Ignore
        }
    }
}

#[test]
fn test_retry_error() {
    let error = RetryError {
        message: "テストエラー".to_string(),
    };

    assert_eq!(error.error_action(), ErrorAction::Retry);
}

#[test]
fn test_ignore_error() {
    let error = IgnoreError {
        message: "テストエラー".to_string(),
    };

    assert_eq!(error.error_action(), ErrorAction::Ignore);
}

#[test]
fn test_dynamic_error_retry() {
    let error = DynamicError {
        message: "テストエラー".to_string(),
        should_retry: true,
    };

    assert_eq!(error.error_action(), ErrorAction::Retry);
}

#[test]
fn test_dynamic_error_ignore() {
    let error = DynamicError {
        message: "テストエラー".to_string(),
        should_retry: false,
    };

    assert_eq!(error.error_action(), ErrorAction::Ignore);
}

#[test]
fn test_error_action_debug() {
    assert_eq!(format!("{:?}", ErrorAction::Retry), "Retry");
    assert_eq!(format!("{:?}", ErrorAction::Ignore), "Ignore");
}

#[test]
fn test_error_action_clone() {
    let action1 = ErrorAction::Retry;
    let action2 = action1.clone();

    assert_eq!(action1, action2);

    let action3 = ErrorAction::Ignore;
    let action4 = action3.clone();

    assert_eq!(action3, action4);
}

#[test]
fn test_error_action_eq() {
    assert_eq!(ErrorAction::Retry, ErrorAction::Retry);
    assert_eq!(ErrorAction::Ignore, ErrorAction::Ignore);
    assert_ne!(ErrorAction::Retry, ErrorAction::Ignore);
}
