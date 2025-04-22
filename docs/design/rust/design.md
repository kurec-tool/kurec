# Rust 設計ルール

## 🎯 主要要件 (docs/overview.md より)

- **高可用性:** ジョブ失敗時の自動再試行、DLQでの永久障害切り分け。
- **スケーラビリティ:** 水平分散型アーキテクチャを採用。
- **型安全性:** `#[define_event_stream]` マクロと `Event` トレイトでメッセージ型とストリーム名をコンパイル時に整合させる。
- **一貫性:** `EventStream` によるストリーム定義の一元管理、`setup_all_streams` で起動時プロビジョニング。
- **テスト容易性:** マクロの `trybuild`、JetStream E2E、in-memory テストを組み合わせたテスト戦略を採用。
- **可観測性:** ミドルウェア層でエラー分類・ロギング・メトリクスを一元化。

## 🏛 アーキテクチャレイヤー (docs/design.md より)

- 依存関係は以下の通りとする:
    - `shared-macros` -> なし
    - `shared-core` -> なし
    - `shared-types` -> なし
    - `infra_macros` -> なし
    - `domain` -> `shared-core`, `shared-types`
    - `infra` -> `domain`, `shared-core`, `shared-types`, `infra_macros`
    - `app_macros` -> なし
    - `app (workers)` -> `domain`, `infra`, `app_macros`

## ⚙️ コア機能とコード生成 (docs/design.md より)

- `shared-core` (`shared-types` を含む): コア機能（エラーハンドリング、KVSバケット関連トレイト (`KvsBucket`)、ワーカー構築）を提供する。
- `shared-macros`: 一部のコード生成（`define_kvs_bucket!`, `#[worker]`）を担当する。
- `domain`: ドメインモデルとユースケース、およびイベント関連トレイト (`Event`) を提供する。
- `infra_macros`: インフラ層のコード生成（`#[define_event_stream]`）を担当する。`StreamAttributes`構造体を定義し、`domain`クレートが`infra_jetstream`に依存せずにイベントストリームの設定を行えるようにする。
- `app_macros`: アプリケーション層のコード生成を担当する。

## 📦 infra と app (docs/design.md より)

- `infra`: 外部システムとの接続や具体的な実装を担当するクレート群。
  - `infra_nats`: NATS サーバーへの接続と、JetStream コンテキストや KV ストアへの基本的なアクセスを提供する。
  - `infra_jetstream`: `infra_nats` を利用し、JetStream の Pub/Sub 機能（`JsPublisher`, `JsSubscriber`）やストリーム管理機能 (`setup_all_streams`) を提供する。`EventStream`クラスを通じてイベントストリームの設定を管理する。`StreamConfig`構造体を定義し、`StreamAttributes`から変換して使用する。
  - `infra_kvs`: `infra_nats` を利用し、NATS KV ストアを用いたリポジトリ実装 (`NatsKvProgramRepository`) を提供する。
  - `infra_mirakc`: mirakc API クライアントや SSE イベントソースを提供する。
  - (その他、必要に応じて `infra_*` クレートを追加)
- `app (workers)`: `domain` と `infra` を組み合わせて具体的なワーカーアプリケーションを構築する。CLI (`clap`) でワーカーを選択可能にする。`app_macros`を使用してイベントストリームの定義を行う。

## 🔄 エラーハンドリング (docs/design.md より)

- `ClassifyError` トレイトと `ErrorAction` (Retry/Ignore) を使用してエラーを分類する。
- ミドルウェア層でエラー分類に基づき retry/ack を決定する。
- DLQ は専用パイプラインまたは手動再投入で処理する。

## 🚀 開発フロー (docs/design.md より)

- 原則としては以下の順序で開発を進める
- ただし、作業を細かく分割出来ると分かった場合はそちらを優先する
  - 作業を分割した場合は、分割した作業のテストを作成して実行する
  - テストが通ったらgitにコミットする
- 開発フローの各段階でテストコードを作成し、テストを実行することで手戻りを防ぐ
- 各段階でテストを作成して実行し、成功するまで次に進まない
- 順序:
    1. マクロ定義 (`shared-macros`, `infra_macros`, `app_macros`)
    2. ポート定義 (`domain/ports`)
    3. ドメインユースケース (`domain`)
    4. infra 実装 (`infra`)
    5. app ワーカー (`app`)

## 📊 DTOとドメインモデル (docs/design.md より)

- DTO は `shared/core/src/dtos/`, `infra/{実装名}/src/dtos/`, `domain/src/dtos/` に配置する。
- リポジトリ実装内で外部APIレスポンス等を共通DTOに変換する。
- ユースケース内で共通DTOをドメインモデルに変換する。

## 🏗 ユースケースとリポジトリ (docs/design.md より)

- ユースケースは `domain/src/usecases/` に配置し、リポジトリインターフェースに依存する。
- リポジトリインターフェースは `domain/src/ports/repositories/` に定義する。
- リポジトリ実装は `infra/{実装名}/src/repositories/` に配置し、インターフェースを実装する。
- ユースケースとリポジトリ実装には適切なテスト（モック、インテグレーション）を作成する。

## 🔄 イベントストリームの定義と使用

- `Event`トレイトは`domain/src/event.rs`に定義され、イベント型の基本的な振る舞いを規定する。
- `infra_macros`クレートの`#[define_event_stream]`マクロを使用して、イベント型にストリーム設定を関連付ける。
  - マクロは`StreamAttributes`構造体を使用して、ストリーム設定を定義する。
  - `StreamAttributes`は`infra_macros`クレートで定義され、`domain`クレートが`infra_jetstream`に依存せずにイベントストリームの設定を行えるようにする。
  - マクロは個別の定数（`STREAM_NAME`, `STREAM_MAX_AGE`, `STREAM_MAX_MSGS`など）を生成し、`infra_jetstream`クレートの`StreamConfig`構造体に変換して使用する。
- `infra_jetstream`クレートの`EventStream`クラスを使用して、ストリーム名と設定を管理する。
- `JsPublisher`と`JsSubscriber`は`EventStream`を受け取り、イベントの発行と購読を行う。
- `app_macros`クレートを使用して、アプリケーション層でイベントストリームの定義を行う。

この設計により、`domain`クレートが`infra_jetstream`に依存することなく、イベント型とストリーム設定を関連付けることができる。循環依存を解消し、より柔軟なアーキテクチャを実現している。
