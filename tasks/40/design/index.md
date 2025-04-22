# Issue #40: Ack/Nack機能の修正設計 - 目次

## 概要

js_subscriberでAck/Nack機能が削除されてしまい、自動的にすべてのメッセージにAckするようになってしまっている問題を修正します。メッセージにAckを返すタイミングはEventHandlerで処理が終わり、SinkのストリームにきちんとPublish終わるまで待ってからにします。

## 設計ドキュメント

1. [概要](overview.md) - 問題の背景と設計方針
2. [インフラ共通層](infra_common.md) - Ackトレイトとイベントソースの設計
3. [JetStream実装](jetstream.md) - JetStreamのAck実装
4. [SSE実装](sse.md) - SSEのAck実装
5. [アプリケーション層](application.md) - StreamWorkerの更新
6. [テスト実装](test.md) - Ack/Nack機能のテスト

## 実装方針

1. インフラ共通層に`Ack`トレイトと`AckableEvent`構造体を定義
2. JetStreamとSSEの実装を更新
3. StreamWorkerを更新して、明示的にAckを送信するように変更
4. テストを追加して、Ack/Nack機能が正しく動作することを確認

## 主な変更点

1. **明示的なAck**:
   - 以前の実装では、自動的にすべてのメッセージにAckしていた
   - 新しい実装では、処理が完了した後に明示的にAckを送信

2. **エラー処理の改善**:
   - エラーアクションに基づいて、Ackするかどうかを決定
   - エラーの詳細をログに記録

3. **処理フローの改善**:
   - Sinkへの発行が完了した後にAckを送信
   - エラーの場合は、エラーアクションに基づいてAckするかどうかを決定

## 実装計画

以下の順序で実装を進めます：

1. **インフラ共通層の実装**
   - Ackトレイト
   - AckableEvent構造体
   - EventSourceトレイト

2. **JetStream実装の更新**
   - JsEventError
   - JsAck
   - JsSubscriberの更新

3. **SSE実装の更新**
   - SseEventError
   - SseAck
   - SseEventSourceの更新

4. **アプリケーション層の更新**
   - EventHandlerトレイト
   - StreamWorkerの更新

5. **テストの実装**
   - Ack/Nack機能のテスト
