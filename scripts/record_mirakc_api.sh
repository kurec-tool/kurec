#!/bin/bash
# mirakcのAPIレスポンスを記録するスクリプト

# スクリプトのあるディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# プロジェクトのルートディレクトリを取得
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# カレントディレクトリをプロジェクトのルートに変更
cd "$ROOT_DIR"

# mirakcサーバーのURLを設定（デフォルトはtuner:40772）
MIRAKC_URL=${MIRAKC_URL:-"http://tuner:40772"}

# レコーディングモードでテストを実行
RECORD=1 MIRAKC_URL="$MIRAKC_URL" cargo test --package infra-mirakc test_get_version_recording

echo "mirakcのAPIレスポンスの記録が完了しました。"
echo "記録されたレスポンスは rust/libs/infra/mirakc/tests/fixtures/ ディレクトリに保存されています。"
