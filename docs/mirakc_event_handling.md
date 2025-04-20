# mirakc イベントハンドリング設計

## 概要

mirakc の `/events` から SSE でイベントを取得し、システム内で扱うための設計。

クリーンアーキテクチャーの原則と、将来的な複数 mirakc サーバー対応を考慮。

## アーキテクチャ

*   **イベント取得元:** mirakc (SSE)
*   **イベント形式:** Server-Sent Events (SSE)
*   **イベント伝播:** JetStream
*   **複数mirakc対応:** 各イベントに mirakc サーバーの識別子を付与

## コンポーネント

### 1. `MirakcEventStreamProvider` (shared-core)

*   **役割:** mirakc サーバーからのイベントストリームを提供するインターフェース。
*   **場所:** `shared_core::ports::mirakc_events`
*   **インターフェース:**

    ```rust
    #[async_trait::async_trait]
    pub trait MirakcEventStreamProvider: Send + Sync + 'static {
        /// 指定されたmirakcサーバーからイベントストリームを取得する
        async fn get_event_stream(
            &self,
            source_id: String, // 設定ファイルで定義される識別子 (例: "main-tuner")
            config: MirakcSourceConfig,
        ) -> Result<impl Stream<Item = Result<MirakcSystemEvent>> + Send>;
    }
    ```

*   **`MirakcSourceConfig`**: mirakc サーバーへの接続情報 (base URL) を保持する構造体。

    ```rust
    pub struct MirakcSourceConfig {
        pub base_url: Url,
        // 必要であれば認証情報なども追加
    }
    ```

### 2. `MirakcSystemEvent` (shared-core)

*   **役割:** システム内部で扱うイベントの形式を定義する。
*   **場所:** `shared_core::dtos::mirakc_event`
*   **構造:**

    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MirakcSystemEvent {
        /// どのmirakcサーバーからのイベントかを示す識別子 (設定ファイル由来)
        pub source_id: String,
        /// イベントのペイロード
        pub payload: MirakcEventPayload,
        /// イベント発生時刻 (受信時刻)
        #[serde(with = "chrono::serde::ts_milliseconds_option")]
        pub timestamp: DateTime<Utc>,
    }
    ```

*   **`MirakcEventPayload`**: mirakc から受信するイベントのペイロードを表現する enum。

    ```rust
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type", content = "data")]
    #[serde(rename_all = "camelCase")]
    pub enum MirakcEventPayload {
        TunerStatusChanged(TunerStatusChanged),
        EpgProgramsUpdated(EpgProgramsUpdated),
        RecordingStarted(RecordingStarted),
        RecordingStopped(RecordingStopped),
        RecordingFailed(RecordingFailed),
        RecordingRescheduled(RecordingRescheduled),
        RecordSaved(RecordSaved),
        RecordBroken(RecordBroken),
        RecordRemoved(RecordRemoved),
        ContentRemoved(ContentRemoved),
        OnairProgramChanged(OnairProgramChanged),
    }
    ```

### 3. `SseMirakcEventStreamProvider` (infra)

*   **役割:** `MirakcEventStreamProvider` インターフェースの実装。SSE で mirakc からイベントを取得する。
*   **場所:** `infra_mirakc_events` クレート (新規)
*   **処理:**
    1.  `MirakcSourceConfig` を使用して mirakc の `/events` エンドポイントに接続。
    2.  SSE イベントを受信し、`MirakcEventPayload` にデシリアライズ。
    3.  `MirakcSystemEvent` を生成し、ストリームとして出力。

### 4. `mirakc_event_poller` (app)

*   **役割:** 設定に基づき複数の mirakc サーバーからイベントを取得し、JetStream に発行する。
*   **場所:** `app::workers::mirakc_event_poller` モジュール (新規)
*   **処理:**
    1.  起動時引数から `source_id` を取得。
    2.  設定ファイルから `source_id` に対応する `MirakcSourceConfig` を取得。
    3.  `SseMirakcEventStreamProvider` を使用してイベントストリームを取得。
    4.  受信した `MirakcSystemEvent` を JetStream に発行。

## 設定

*   `kurec.yml` から `mirakc_sources` セクションを削除。
*   ワーカーごとの設定は、環境変数またはコマンドライン引数で指定。

## イベント定義

*   `MirakcEventPayload` の各バリアントは、mirakc の `/events` API で定義されているイベントに対応。
*   各イベントの詳細は、mirakc のソースコード (events.rs) を参照。

## 今後の課題

*   `DateTime<Utc>` のシリアライズ・デシリアライズ方法の検討 (今回は `chrono::serde::ts_milliseconds_option` を使用)。
*   エラーハンドリング戦略の検討。
*   テストコードの作成。
