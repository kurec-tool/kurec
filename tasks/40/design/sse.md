# Issue #40: Ack/Nack機能の修正設計 - SSE実装

SSE実装では、Ackトレイトの実装とSseEventSourceの更新を行います。

## SseEventError

```rust
// infra/mirakc/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SseEventError {
    #[error("Failed to deserialize event: {source}")]
    Deserialize {
        #[source]
        source: serde_json::Error,
        payload: Vec<u8>,
    },
    
    #[error("Connection error: {source}")]
    Connection {
        #[source]
        source: reqwest::Error,
        endpoint: String,
    },
    
    #[error("SSE stream error: {source}")]
    Stream {
        #[source]
        source: eventsource_stream::Error,
    },
}

impl SseEventError {
    pub fn should_retry(&self) -> bool {
        match self {
            Self::Deserialize { .. } => false,
            Self::Connection { .. } => true,
            Self::Stream { .. } => true,
        }
    }
    
    pub fn log(&self) {
        use tracing::{error, warn};
        
        match self {
            Self::Deserialize { payload, .. } => {
                error!(
                    error = %self,
                    payload_size = payload.len(),
                    payload_preview = %String::from_utf8_lossy(&payload[..std::cmp::min(100, payload.len())]),
                    "Failed to deserialize SSE event"
                );
            },
            Self::Connection { endpoint, .. } => {
                warn!(
                    error = %self,
                    endpoint = %endpoint,
                    "SSE connection error"
                );
            },
            Self::Stream { .. } => {
                warn!(
                    error = %self,
                    "SSE stream error"
                );
            },
        }
    }
}
```

## SseAck

```rust
// infra/mirakc/src/ack.rs
use async_trait::async_trait;
use anyhow::Result;
use infra_common::ack::Ack;

/// SSE用のAck実装
pub struct SseAck {}

impl SseAck {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Ack for SseAck {
    async fn ack(&self) -> Result<()> {
        // SSEはAck機能がないので何もしない
        Ok(())
    }
}
```

## SseEventSource

```rust
// infra/mirakc/src/mirakc_sse_source.rs
use crate::ack::SseAck;
use crate::error::SseEventError;
use anyhow::Result;
use async_trait::async_trait;
use domain::event::Event;
use eventsource_stream::Eventsource;
use futures::stream::{BoxStream, StreamExt, TryStreamExt};
use infra_common::ackable_event::AckableEvent;
use infra_common::event_source::EventSource;
use reqwest::Client;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{debug, error, info};

/// SSEを使用したイベント購読者
pub struct SseEventSource<E: Event> {
    client: Client,
    sse_url: String,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: Event> SseEventSource<E> {
    /// 新しいSseEventSourceを作成
    pub fn new(client: Client, sse_url: String) -> Self {
        Self {
            client,
            sse_url,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<E> EventSource<E, SseEventError> for SseEventSource<E>
where
    E: Event + Debug + Send + Sync + 'static,
{
    async fn subscribe(&self) -> Result<BoxStream<'static, Result<AckableEvent<E>, SseEventError>>> {
        let sse_url = self.sse_url.clone();
        
        info!(url = %sse_url, "Connecting to SSE endpoint");
        
        let response = self.client.get(&sse_url)
            .send()
            .await
            .map_err(|e| {
                let error = SseEventError::Connection {
                    source: e,
                    endpoint: sse_url.clone(),
                };
                error.log();
                anyhow::anyhow!("Failed to connect to SSE endpoint: {}", error)
            })?;
        
        let status = response.status();
        if !status.is_success() {
            let error = SseEventError::Connection {
                source: reqwest::Error::from(reqwest::StatusCode::from_u16(status.as_u16()).unwrap_err()),
                endpoint: sse_url.clone(),
            };
            error.log();
            return Err(anyhow::anyhow!("Failed to connect to SSE endpoint: {}", error));
        }
        
        let sse_stream = response
            .bytes_stream()
            .eventsource()
            .map_err(move |e| {
                // SSEストリームのエラー
                let error = SseEventError::Stream {
                    source: e,
                };
                
                error.log();
                
                error
            })
            .and_then(move |event| async move {
                match serde_json::from_str::<E>(&event.data) {
                    Ok(parsed_event) => {
                        // 成功結果を返す
                        let ack_fn = Box::new(SseAck::new());
                        Ok(AckableEvent::new(parsed_event, ack_fn))
                    }
                    Err(e) => {
                        // デシリアライズエラー結果を返す
                        let error = SseEventError::Deserialize {
                            source: e,
                            payload: event.data.as_bytes().to_vec(),
                        };
                        
                        error.log();
                        
                        Err(error)
                    }
                }
            });
            
        Ok(Box::pin(sse_stream))
    }
}
```

## 設計のポイント

1. **SseAck**:
   - SSEはAck機能がないため、何もしない実装
   - `ack`メソッドを呼び出しても、何も行わない

2. **SseEventError**:
   - SSE関連のエラーを表現する列挙型
   - `should_retry`メソッドで、エラーが再試行可能かどうかを判断
   - `log`メソッドで、エラーの詳細をログに記録

3. **SseEventSource**:
   - SSEからのイベントを購読するためのクラス
   - `subscribe`メソッドで、AckableEventを返すストリームを提供
   - デシリアライズエラーの場合はエラーを返す（JetStreamと異なり、Ackする必要がない）

## SSEとJetStreamの違い

SSEとJetStreamの主な違いは以下の通りです：

1. **Ack機能**: JetStreamはAck機能を持ち、クライアントがメッセージを正常に処理したことを確認できます。SSEにはAck機能がありません。

2. **メッセージの再配信**: JetStreamは未確認のメッセージを再配信できますが、SSEは再接続時に未受信のイベントを取得するだけです。

3. **メッセージの永続化**: JetStreamはメッセージを永続化し、後で取得できますが、SSEはリアルタイムのイベントストリームのみを提供します。

4. **スケーラビリティ**: JetStreamは複数のコンシューマーにメッセージを配信できますが、SSEは1対1の接続です。

これらの違いにより、SSEの場合はAckが実質的に何も行わない実装になっています。しかし、統一的なインターフェースを提供するために、JetStreamと同じようにAckableEventを使用しています。
