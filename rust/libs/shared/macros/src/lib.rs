use proc_macro::TokenStream;
#[cfg(test)]
use std::time::Duration;

mod define_streams;
mod event;
mod stream_worker;

/// ストリームワーカーを定義するマクロ
///
/// このマクロは、イベント処理関数をラップして、JetStreamを使用したストリームワーカーを生成します。
/// 元の関数をそのまま保持し、さらに `{関数名}_worker` という名前のワーカー実行関数を生成します。
///
/// # 要件
///
/// - 関数は正確に1つのパラメータを持つ必要があります
/// - 関数は `Result<OutputType, ErrorType>` を返す必要があります
/// - パラメータと戻り値の型は `Event` トレイトを実装している必要があります
/// - エラー型は `ClassifyError` トレイトを実装している必要があります
///
/// # 属性パラメータ
///
/// - `durable = "名前"`: 永続的なコンシューマー名を指定します（省略可）
///
/// # 生成される関数
///
/// このマクロは元の関数に加えて、以下の関数を生成します：
///
/// ```text
/// async fn 元の関数名_worker(
///     js_ctx: &infra_jetstream::JetStreamCtx,
///     shutdown: tokio_util::sync::CancellationToken
/// ) -> anyhow::Result<()>
/// ```
///
/// # 使用例
///
/// ```ignore
/// use shared_macros::{event, stream_worker};
/// use serde::{Serialize, Deserialize};
/// use shared_core::error_handling::{ClassifyError, ErrorAction};
///
/// // エラー型定義
/// #[derive(Debug, thiserror::Error)]
/// enum MyError {
///     #[error("エラー: {0}")]
///     Error(String),
/// }
///
/// // エラー分類の実装
/// impl ClassifyError for MyError {
///     fn error_action(&self) -> ErrorAction {
///         ErrorAction::Retry
///     }
/// }
///
/// #[event(stream = "test")]
/// #[derive(Serialize, Deserialize)]
/// struct InputEvent {
///     data: String,
/// }
///
/// #[event(stream = "test")]
/// #[derive(Serialize, Deserialize)]
/// struct OutputEvent {
///     data: String,
/// }
///
/// #[stream_worker]
/// async fn process_event(event: InputEvent) -> Result<OutputEvent, MyError> {
///     Ok(OutputEvent { data: event.data })
/// }
///
/// // 以下の関数が生成されます：
/// // async fn process_event_worker(
/// //     js_ctx: &infra_jetstream::JetStreamCtx,
/// //     shutdown: tokio_util::sync::CancellationToken
/// // ) -> anyhow::Result<()>
/// ```
#[proc_macro_attribute]
pub fn stream_worker(attr: TokenStream, item: TokenStream) -> TokenStream {
    stream_worker::stream_worker_impl(attr, item)
}

#[proc_macro]
pub fn define_streams(input: TokenStream) -> TokenStream {
    define_streams::define_streams_impl(input)
}

#[cfg(test)]
fn parse_duration(opt: &Option<String>) -> Option<Duration> {
    opt.as_ref().map(|s| {
        humantime::parse_duration(s)
            .unwrap_or_else(|e| panic!("invalid duration literal `{}`: {}", s, e))
    })
}

#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::event_impl(attr, item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parse_duration() {
        assert_eq!(
            parse_duration(&Some("1s".to_string())),
            Some(Duration::new(1, 0))
        );
        assert_eq!(
            parse_duration(&Some("1m".to_string())),
            Some(Duration::new(60, 0))
        );
        assert_eq!(
            parse_duration(&Some("1h".to_string())),
            Some(Duration::new(3600, 0))
        );
        assert_eq!(parse_duration(&None), None);
    }
}
