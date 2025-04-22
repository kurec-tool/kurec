# Issue #40: Ack/Nack機能の修正設計 - インフラ共通層

インフラ層に共通のクレート（infra_common）を作成し、そこにAckトレイトとAckableEvent構造体を定義します。

## Ackトレイト

```rust
// infra/common/src/ack.rs
use anyhow::Result;
use async_trait::async_trait;

/// イベントのAck（確認応答）機能を提供するトレイト
#[async_trait]
pub trait Ack: Send + Sync + 'static {
    /// イベントを確認応答（Ack）する
    async fn ack(&self) -> Result<()>;
}
```

## AckableEvent構造体

```rust
// infra/common/src/ackable_event.rs
use anyhow::Result;
use domain::event::Event;
use std::fmt::Debug;

use crate::ack::Ack;

/// イベントとAck機能をラップする構造体
pub struct AckableEvent<E: Event> {
    event: E,
    ack: Option<Box<dyn Ack>>,
}

impl<E: Event> AckableEvent<E> {
    /// 新しいAckableEventを作成
    pub fn new(event: E, ack: Box<dyn Ack>) -> Self {
        Self {
            event,
            ack: Some(ack),
        }
    }

    /// イベントを取得
    pub fn event(&self) -> &E {
        &self.event
    }

    /// イベントを消費して取得
    pub fn into_event(self) -> E {
        self.event
    }

    /// 明示的にAckを送信
    pub async fn ack(&mut self) -> Result<()> {
        if let Some(ack) = self.ack.take() {
            ack.ack().await
        } else {
            Ok(()) // 既にAckされている場合は何もしない
        }
    }

    /// Ackが可能かどうかを確認
    pub fn can_ack(&self) -> bool {
        self.ack.is_some()
    }
}

impl<E: Event + Debug> Debug for AckableEvent<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AckableEvent")
            .field("event", &self.event)
            .field("ack", &if self.ack.is_some() { "Some" } else { "None" })
            .finish()
    }
}

impl<E: Event> AsRef<E> for AckableEvent<E> {
    fn as_ref(&self) -> &E {
        &self.event
    }
}
```

## EventSourceトレイト

```rust
// infra/common/src/event_source.rs
use anyhow::Result;
use async_trait::async_trait;
use domain::event::Event;
use futures::stream::BoxStream;

use crate::ackable_event::AckableEvent;

/// イベントソースのトレイト
#[async_trait]
pub trait EventSource<E, Err>: Send + Sync + 'static
where
    E: Event,
    Err: Send + Sync + 'static,
{
    /// イベントを購読する
    async fn subscribe(
        &self,
    ) -> Result<
        BoxStream<'static, Result<AckableEvent<E>, Err>>,
    >;
}
```

## 設計のポイント

1. **AckableEvent構造体**:
   - イベントとAck機能をラップする構造体
   - `ack`メソッドを明示的に呼び出すことで、Ackのタイミングを制御できる
   - `Option<Box<dyn Ack>>`を使用して、Ackが一度だけ呼び出されることを保証
   - `&mut self`を取るため、所有権システムによりAckの呼び出しを制御

2. **Ackトレイト**:
   - Ack機能を抽象化するトレイト
   - 異なるメッセージングシステム（JetStream、SSEなど）に対して、適切なAck実装を提供できる

3. **EventSourceトレイト**:
   - イベントソースを抽象化するトレイト
   - AckableEventを返すストリームを提供
