# mirakcインフラ層のテスト実行方法

このドキュメントでは、mirakcインフラ層のテストの実行方法について説明します。

## テストの種類

mirakcインフラ層には、以下の3種類のテストがあります：

1. **単体テスト**: wiremockを使用したモックサーバーでのテスト
2. **レコーディングテスト**: 実際のAPIレスポンスを記録して再生するテスト
3. **実際のmirakcサーバーを使用したテスト**: 実際のmirakcサーバーに接続してテストを実行

## テスト実行用スクリプト

テストを簡単に実行するために、以下のスクリプトが用意されています：

- `scripts/test_mirakc_infra.sh`: 単体テスト実行用
- `scripts/record_mirakc_api.sh`: レコーディングテスト実行用
- `scripts/test_mirakc_real.sh`: 実際のmirakcサーバーを使用したテスト実行用

これらのスクリプトは、プロジェクトのルートディレクトリから実行することができます。

## 単体テストの実行

単体テストは、wiremockを使用したモックサーバーでのテストです。実際のmirakcサーバーに接続せずにテストを実行することができます。

```bash
./scripts/test_mirakc_infra.sh
```

## レコーディングテストの実行

レコーディングテストは、実際のAPIレスポンスを記録して再生するテストです。初回実行時または`RECORD=1`環境変数を設定した場合は、実際のmirakcサーバーに接続してレスポンスを記録します。2回目以降の実行時は、記録されたレスポンスを使用してテストを実行します。

```bash
# デフォルトのmirakcサーバー（http://tuner:40772）を使用
./scripts/record_mirakc_api.sh

# カスタムのmirakcサーバーを使用
MIRAKC_URL="http://custom-server:40772" ./scripts/record_mirakc_api.sh
```

記録されたレスポンスは、`rust/libs/infra/mirakc/tests/fixtures/`ディレクトリに保存されます。

## 実際のmirakcサーバーを使用したテストの実行

実際のmirakcサーバーを使用したテストは、実際のmirakcサーバーに接続してテストを実行します。このテストは、デフォルトでは`#[ignore]`属性が付いているため、通常のテスト実行では実行されません。

```bash
# デフォルトのmirakcサーバー（http://tuner:40772）を使用
./scripts/test_mirakc_real.sh

# カスタムのmirakcサーバーを使用
MIRAKC_URL="http://custom-server:40772" ./scripts/test_mirakc_real.sh
```

## 手動でのテスト実行

スクリプトを使用せずに、直接`cargo test`コマンドを使用してテストを実行することもできます：

```bash
# 単体テスト
cargo test --package infra-mirakc

# レコーディングテスト
RECORD=1 cargo test --package infra-mirakc test_get_version_recording

# 実際のmirakcサーバーを使用したテスト
cargo test --package infra-mirakc -- --ignored
```

## 環境変数

テスト実行時に設定できる環境変数：

- `MIRAKC_URL`: mirakcサーバーのURL（デフォルト: `http://tuner:40772`）
- `RECORD`: レコーディングモードを有効にするフラグ（値は何でも良い、例: `RECORD=1`）
