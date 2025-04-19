#!/bin/bash
# 実際のmirakcサーバーを使用したテストを実行するスクリプト

# スクリプトのあるディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# プロジェクトのルートディレクトリを取得
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# カレントディレクトリをプロジェクトのルートに変更
cd "$ROOT_DIR"

# mirakcサーバーのURLを設定（デフォルトはtuner:40772）
MIRAKC_URL=${MIRAKC_URL:-"http://tuner:40772"}

# 実際のmirakcサーバーを使用したテストを実行
MIRAKC_URL="$MIRAKC_URL" cargo test --package infra-mirakc -- --ignored

echo "実際のmirakcサーバー（$MIRAKC_URL）を使用したテストが完了しました。"
