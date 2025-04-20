# mirakcイベント処理

このドキュメントでは、mirakcから送信されるイベントの取得と処理方法について説明します。

## 概要

mirakcは `/events` エンドポイントを通じて、Server-Sent Events (SSE) 形式でさまざまなイベントを送信します。これらのイベントには、チューナーのステータス変更、EPGプログラムの更新、録画の開始・停止などが含まれます。

本システムでは、これらのイベントをリアルタイムで取得し、JetStreamを通じてシステム内の他のコンポーネントに配信します。

## アーキテクチャ

mirakcイベント処理は、クリーンアーキテクチャに基づいて以下のコンポーネントで構成されています：

### 1. インターフェース層

- `MirakcEventRepository` - mirakcイベントを取得するためのリポジトリインターフェース
  - `get_event_stream()` - イベントストリームを取得するメソッド

- `EventPublisher` - イベントを発行するためのインターフェース
  - `publish(event)` - イベントを発行するメソッド

### 2. ドメイン層

- mirakcイベント型
  - `TunerStatusChangedEvent`
  - `EpgProgramsUpdatedEvent`
  - `RecordingStartedEvent`
  - `RecordingStoppedEvent`
  - など

- `MirakcEventUseCase` - mirakcイベントを処理するためのユースケース
  - `process_events()` - イベントを取得し、適切な型に変換してパブリッシュするメソッド

### 3. インフラ層

- `MirakcEventRepositoryImpl` - SSEを使用してmirakcイベントを取得する実装
  - `get_event_stream()` - SSEストリームを取得し、MirakcEventDtoストリームに変換

- `CombinedPublisher` - JetStreamを使用してイベントを発行する実装
  - 各イベント型に対する `EventPublisher` の実装

### 4. アプリケーション層

- `mirakc_events` コマンド - mirakcイベントを処理するためのCLIコマンド
  - `run_mirakc_events()` - リポジトリ、パブリッシャー、ユースケースを作成し、イベント処理を開始

## イベント処理フロー

1. `MirakcEventRepositoryImpl` が mirakcの `/events` エンドポイントに接続し、SSEストリームを取得
2. SSEストリームを `MirakcEventDto` ストリームに変換
3. `MirakcEventUseCase` が `MirakcEventDto` を受け取り、イベントタイプに応じて適切なイベント型に変換
4. 変換されたイベントを `CombinedPublisher` を通じてJetStreamに発行
5. JetStreamに発行されたイベントは、他のコンポーネントで購読・処理される

## シャットダウン処理

イベント処理は継続的に行われるため、適切なシャットダウン処理が重要です。本システムでは、以下の方法でシャットダウンを実現しています：

1. `CancellationToken` を使用したシグナル伝播
   - アプリケーションのメインスレッドでCTRL+Cシグナルを捕捉
   - シグナル受信時に `CancellationToken` をキャンセル
   - `CancellationToken` を各ワーカーに渡し、シャットダウン時に処理を終了

2. `MirakcEventUseCase` でのシャットダウン処理
   - `with_shutdown()` メソッドで `CancellationToken` を設定
   - `process_events()` メソッド内で `tokio::select!` を使用して、イベント処理とシャットダウンシグナルを同時に待機
   - シャットダウンシグナル受信時にループを終了

## 複数のmirakcサーバー対応

将来的に複数のmirakcサーバーを扱う場合、以下の点に注意が必要です：

1. イベントにmirakc_urlを含める
   - どのサーバーからのイベントかを識別するため
   - 後続の処理で同じmirakcサーバーにアクセスするため

2. サーバーごとに個別のイベント処理ワーカーを起動
   - 各サーバーのイベントを独立して処理
   - サーバーごとに異なるmirakc_urlを設定

## エラーハンドリング

mirakcイベント処理では、以下のエラーハンドリング戦略を採用しています：

1. 接続エラー
   - 指数バックオフを使用した再接続
   - 最大再試行回数なし（無限に再試行）

2. イベントパース・処理エラー
   - エラーログを出力
   - 処理を継続（1つのイベントの失敗が全体に影響しないよう）

## 使用例

```rust
// mirakcイベント処理コマンドの実行
let app_config = AppConfig { nats_url: "nats://localhost:4222".to_string() };
let mirakc_url = "http://localhost:40772";
let shutdown = CancellationToken::new();

// イベント処理を開始
if let Err(e) = cmd::mirakc_events::run_mirakc_events(&app_config, mirakc_url, shutdown.clone()).await {
    eprintln!("mirakc events worker error: {}", e);
}
