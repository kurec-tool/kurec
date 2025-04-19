# Rust テスト ルール

## ✅ 基本方針

- Rustのソースコードを修正したら、必ずテストコードを追加してください。
- 追加・修正したテストを含め、`cargo test` を実行し、すべてのテストが成功することを確認してください。

## 🧪 テスト戦略 (docs/overview.md, docs/design.md より)

- 以下のテスト手法を組み合わせてテスト容易性を確保します:
    - **trybuild**: マクロのコンパイル時テスト (`shared-macros`)
    - **in-process / testcontainers**: JetStreamなどのインフラ層とのE2Eテスト
    - **in-memory**: 依存関係をモックした単体テスト
    - **wiremock**: HTTPクライアント (例: `infra-mirakc`) の単体テスト (モックサーバー)
    - **レコーディングテスト**: 実際のAPIレスポンスを記録・再生する統合テスト (例: `infra-mirakc`)

## 🧪 mirakc-infra テスト戦略 (docs/mirakc_infra_test_strategy.md, docs/mirakc_infra_test.md より)

- `infra-mirakc` クレートのテストは以下の3レベルで実施します:
    1.  **単体テスト**:
        - `wiremock` を使用したモックサーバーでのテスト。
        - CI環境でも実行可能。
        - 実行: `cargo test --package infra-mirakc` または `./scripts/test_mirakc_infra.sh`
    2.  **統合テスト (レコーディングテスト)**:
        - 実際のAPIレスポンスを記録・再生するテスト。
        - CI環境でも実行可能。
        - 記録モード: `RECORD=1 cargo test --package infra-mirakc -- --test-threads=1` または `RECORD=1 ./scripts/record_mirakc_api.sh`
        - 再生モード: `cargo test --package infra-mirakc -- --test-threads=1` または `./scripts/record_mirakc_api.sh` (RECORDなし)
        - レスポンスは `rust/libs/infra/mirakc/tests/fixtures/` に保存されます。
    3.  **オプショナルな手動テスト**:
        - 実際のmirakcサーバーを使用したテスト。
        - 開発環境でのみ実行 (`#[ignore]` を付与)。
        - 実行: `MIRAKC_URL=... cargo test --package infra-mirakc -- --ignored` または `MIRAKC_URL=... ./scripts/test_mirakc_real.sh`
        - 環境変数 `MIRAKC_URL` で接続先を指定 (デフォルト: `http://tuner:40772`)。

## 🧪 ユースケースとリポジトリのテスト (docs/design.md より)

- ユースケース (`domain/src/usecases/`) のテスト:
    - リポジトリインターフェースのモック実装を作成してテストします。
- リポジトリ実装 (`infra/{実装名}/src/repositories/`) のテスト:
    - `wiremock` や `testcontainers` を使用して、外部システムとの連携を含めたインテグレーションテストを実施します。
