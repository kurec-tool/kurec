# クレート命名規則とインポートガイドライン

## クレート命名規則

プロジェクト内のクレート名は以下の規則に従ってください：

### ライブラリクレート

- 形式: `{カテゴリ}_{機能}`（アンダースコア区切り）
- 例:
  - `infra_jetstream`: JetStream関連のインフラストラクチャコード
  - `infra_mirakc`: mirakc関連のインフラストラクチャコード
  - `domain_user`: ユーザードメインのコード
  - `shared_core`: 共有コアライブラリ
  - `shared_macros`: 共有マクロライブラリ

### アプリケーションクレート

- 形式: `{アプリ名}-{コンポーネント}`（ハイフン区切り）
- 例:
  - `kurec-app`: メインアプリケーション
  - `kurec-cli`: コマンドラインインターフェース

### 命名の一貫性について

- **重要**: ライブラリクレートは必ずアンダースコア(`_`)区切りを使用し、アプリケーションクレートはハイフン(`-`)区切りを使用してください
- ハイフン(`-`)とアンダースコア(`_`)を混在させないでください
- Rustではハイフンを含むクレート名をコード内で参照する場合、自動的にアンダースコアに変換されます（例: `use some-crate` → `use some_crate`）
- 混乱を避けるため、ライブラリクレートの命名には最初からアンダースコアを使用してください

## Cargo.tomlでの依存関係の指定

内部依存関係は以下のように指定してください：

```toml
[dependencies]
# 内部依存関係
infra_jetstream = { path = "../libs/infra/jetstream" }
shared_core = { path = "../libs/shared/core" }
shared_macros = { path = "../libs/shared/macros" }
```

## Rustコードでのインポート

クレートのインポートは、Cargo.tomlで指定した名前と完全に一致させてください：

```rust
// 正しいインポート
use infra_jetstream::JsPublisher;
use shared_core::event_metadata::Event;
use shared_macros::stream_worker;

// 誤ったインポート（避けてください）
use shared_infra_jetstream::JsPublisher;  // ❌ 不正確なクレート名
```

## 依存関係の変更

クレート名や公開APIを変更する場合は、以下のステップに従ってください：

1. チームに変更を通知する
2. 変更内容をドキュメント化する
3. 移行ガイドを提供する
4. 十分な移行期間を設ける

## 依存関係の問題を防ぐためのベストプラクティス

1. クレート名を変更する前に、そのクレートを使用しているすべての場所を特定する
2. 新しいAPIを導入する際は、古いAPIとの互換性を維持するか、明確な移行パスを提供する
3. 定期的にプロジェクト全体をビルドして、依存関係の問題を早期に発見する
4. 新しいクレートを追加する際は、既存のクレート命名規則に従う
5. クレート名とファイルパスの一貫性を保つ

## 環境変数の使用

環境変数を使用する場合は、以下のガイドラインに従ってください：

1. 環境変数名は大文字のスネークケースを使用する（例: `NATS_URL`）
2. 環境変数の使用箇所をドキュメント化する
3. 環境変数が設定されていない場合のデフォルト値を提供する
4. 環境変数の検証を行い、無効な値を早期に検出する

例：

```rust
// 環境変数からNATS URLを取得
let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

// 環境変数の検証
if !nats_url.starts_with("nats://") {
    return Err(anyhow::anyhow!("NATS_URL must start with 'nats://'"));
}
```

## APIの変更と互換性

APIを変更する場合は、以下のガイドラインに従ってください：

1. 非互換な変更を行う場合は、バージョン番号のメジャーバージョンを上げる
2. 古いAPIを非推奨（deprecated）としてマークし、新しいAPIへの移行パスを提供する
3. 非推奨APIには`#[deprecated]`属性を使用し、代替手段を明記する

例：

```rust
/// 新しいJsSubscriberを作成
///
/// # 非推奨
///
/// このメソッドは非推奨です。代わりに `from_event_type` を使用してください。
#[deprecated(
    since = "0.2.0",
    note = "このメソッドは非推奨です。代わりに `from_event_type` を使用してください。"
)]
pub fn new(js_ctx: JetStreamCtx, stream_name: &str, stream_subject: &str) -> Self {
    // 実装...
}
```

これらのガイドラインに従うことで、依存関係の問題を事前に検出し、スムーズな開発プロセスを維持することができます。
