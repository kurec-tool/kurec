#!/bin/bash
# mirakcインフラ層の単体テストを実行するスクリプト

# スクリプトのあるディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# プロジェクトのルートディレクトリを取得
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# カレントディレクトリをプロジェクトのルートに変更
cd "$ROOT_DIR"

# 単体テストを実行
cargo test --package infra-mirakc

echo "mirakcインフラ層の単体テストが完了しました。"
