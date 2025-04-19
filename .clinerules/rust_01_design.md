# Rust 設計ルール

## 🎯 主要要件 (docs/overview.md より)

- **高可用性:** ジョブ失敗時の自動再試行、DLQでの永久障害切り分け。
- **スケーラビリティ:** 水平分散型アーキテクチャを採用。
- **型安全性:** `#[event]` マクロと `Event` トレイトでメッセージ型とストリーム名をコンパイル時に整合させる。
- **一貫性:** `StreamDef` によるストリーム定義の一元管理、`setup_all_streams` で起動時プロビジョニング。
- **テスト容易性:** マクロの `trybuild`、JetStream E2E、in-memory テストを組み合わせたテスト戦略を採用。
- **可観測性:** ミドルウェア層でエラー分類・ロギング・メトリクスを一元化。

## 🏛 アーキテクチャレイヤー (docs/design.md より)

- 依存関係は以下の通りとする:
    - `shared-macros` -> なし
    - `shared-core` -> なし
    - `domain` -> `shared-core`, `shared-macros`
    - `infra` -> `shared-core`, `shared-macros`
    - `app (workers)` -> `domain`, `infra`

## ⚙️ コア機能とコード生成 (docs/design.md より)

- `shared-core`: コア機能（エラーハンドリング、イベント関連トレイト、ストリーム設定、ワーカー構築）を提供する。
- `shared-macros`: コード生成（`#[event]`, `define_streams!`, `#[worker]`）を担当する。

## 📦 infra と app (docs/design.md より)

- `infra`: 外部システム（例: JetStream）との接続を担当する実装を提供する。
- `app (workers)`: `WorkerBuilder` を使用してワーカーを構築し、ミドルウェアを適用する。CLI (`clap`) でワーカーを選択可能にする。

## 🔄 エラーハンドリング (docs/design.md より)

- `ClassifyError` トレイトと `ErrorAction` (Retry/Ignore) を使用してエラーを分類する。
- ミドルウェア層でエラー分類に基づき retry/ack を決定する。
- DLQ は専用パイプラインまたは手動再投入で処理する。

## 🚀 開発フロー (docs/design.md より)

- 以下の順序で開発を進める:
    1. マクロ定義 (`shared-macros`)
    2. ポート定義 (`shared-core/ports`)
    3. ドメインユースケース (`domain`)
    4. infra 実装 (`infra`)
    5. app ワーカー (`app`)
    6. テスト (trybuild, in-process/testcontainers, in-memory)

## 📊 DTOとドメインモデル (docs/design.md より)

- DTO は `shared/core/src/dtos/`, `infra/{実装名}/src/dtos/`, `domain/src/dtos/` に配置する。
- リポジトリ実装内で外部APIレスポンス等を共通DTOに変換する。
- ユースケース内で共通DTOをドメインモデルに変換する。

## 🏗 ユースケースとリポジトリ (docs/design.md より)

- ユースケースは `domain/src/usecases/` に配置し、リポジトリインターフェースに依存する。
- リポジトリインターフェースは `domain/src/ports/repositories/` に定義する。
- リポジトリ実装は `infra/{実装名}/src/repositories/` に配置し、インターフェースを実装する。
- ユースケースとリポジトリ実装には適切なテスト（モック、インテグレーション）を作成する。
