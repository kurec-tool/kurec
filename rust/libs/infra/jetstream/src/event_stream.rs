//! JetStreamイベントストリーム定義
//!
//! このモジュールは、イベント型とJetStreamストリームの設定情報を関連付けるための型を定義します。

use crate::config::StreamConfig;
use domain::event::Event;
use std::marker::PhantomData;

/// イベント型とJetStreamストリームの設定情報を関連付ける構造体
///
/// この構造体は、イベント型に対応するJetStreamストリームの名前と設定情報を保持します。
/// アプリケーション層で、具体的なイベント型に対応するEventStreamのインスタンスを定義します。
#[derive(Debug, Clone)]
pub struct EventStream {
    /// ストリーム名
    stream_name: &'static str,
    /// ストリーム設定
    config: StreamConfig,
}

impl EventStream {
    /// 新しいEventStreamを作成
    ///
    /// # 引数
    ///
    /// * `stream_name` - ストリーム名
    /// * `config` - ストリーム設定
    ///
    /// # 戻り値
    ///
    /// 新しいEventStreamインスタンス
    pub const fn new(stream_name: &'static str, config: StreamConfig) -> Self {
        Self {
            stream_name,
            config,
        }
    }

    /// ストリーム名を取得
    pub fn stream_name(&self) -> &'static str {
        self.stream_name
    }

    /// ストリーム設定を取得
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }
}
