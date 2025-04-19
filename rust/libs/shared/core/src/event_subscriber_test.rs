use crate::event_subscriber::AckHandle;
use anyhow::Result;
use futures::future::BoxFuture;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_ack_handle() -> Result<()> {
    // テスト用のフラグ
    let ack_called = Arc::new(AtomicBool::new(false));
    let ack_called_clone = ack_called.clone();

    // AckHandleを作成
    let ack_fn = Box::new(move || -> BoxFuture<'static, Result<()>> {
        let flag = ack_called_clone.clone();
        Box::pin(async move {
            flag.store(true, Ordering::SeqCst);
            Ok(())
        })
    });

    let ack_handle = AckHandle::new(ack_fn);

    // ackを呼び出す
    ack_handle.ack().await?;

    // フラグが設定されたことを確認
    assert!(ack_called.load(Ordering::SeqCst));

    Ok(())
}

#[tokio::test]
async fn test_ack_handle_error() -> Result<()> {
    // エラーを返すAckHandleを作成
    let ack_fn = Box::new(move || -> BoxFuture<'static, Result<()>> {
        Box::pin(async move { Err(anyhow::anyhow!("Ack error")) })
    });

    let ack_handle = AckHandle::new(ack_fn);

    // ackを呼び出し、エラーが返されることを確認
    let result = ack_handle.ack().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Ack error");

    Ok(())
}
