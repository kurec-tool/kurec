//! イベントとAck機能をラップする構造体を提供するモジュール

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
    ///
    /// # Arguments
    /// * `event` - ラップするイベント
    /// * `ack` - Ack機能を提供するオブジェクト
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
    ///
    /// # Returns
    /// - `Ok(())`: Ackが成功した場合
    /// - `Err(e)`: Ackが失敗した場合
    ///
    /// # Note
    /// このメソッドは一度だけ呼び出すことができます。
    /// 2回目以降の呼び出しでは何も行わず `Ok(())` を返します。
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
