# システム設計概要

以下は 全体アーキテクチャと主要コンポーネントのまとめです。

---

## 🏛 アーキテクチャレイヤー

以下の表は、各レイヤーの依存関係を示しています。

| レイヤー         | 依存先                                         |
|-----------------|-----------------------------------------------|
| **shared-macros** | なし                                          |
| **shared-core**   | なし                                          |
| **domain**        | shared-core, shared-macros (マクロ生成のため)   |
| **infra**         | shared-core, shared-macros                    |
| **app (workers)** | domain, infra                                 |

---

## ⚙️ shared-core（コア機能）

```
shared/core/
├─ src/
│  ├── error_handling.rs       ← `pub trait ClassifyError`, `ErrorAction`
│  ├── event_metadata.rs       ← `pub trait Event`
│  ├── event_publisher.rs      ← `pub trait EventPublisher`
│  ├── event_subscriber.rs     ← `pub trait EventSubscriber`, `AckHandle`
│  ├── streams.rs              ← `StreamConfig`, ストリーム設定の登録・取得
│  ├── worker.rs               ← `WorkerBuilder`, `Middleware`, `Handler`
│  └── stream_worker.rs        ← `StreamWorker`, `StreamMiddleware`, `StreamHandler`
```

- **`Event`**: 全ての `#[event]` 型が実装するトレイト（`stream_name`と`event_name`を提供）
- **`streams`**: ストリーム設定の登録・取得機能
- **`EventPublisher`／`EventSubscriber`**: 入出力の抽象ポート
- **`ClassifyError`／`ErrorAction`**: エラー分類と処理方法の決定
- **`WorkerBuilder`／`StreamWorker`**: ワーカーの構築と実行

---

## 🛠 shared-macros（コード生成）

```
shared/macros/  (proc‑macro = true)
├─ src/lib.rs
│  ├─ #[event(stream=…)]      → Event トレイト実装
│  ├─ define_streams!{...}    → ストリーム設定の登録
│  └─ #[worker(...)]          → WorkerDef 登録＋属性パース
├─ src/define_streams.rs      ← ストリーム定義マクロの実装
└─ src/stream_worker.rs       ← ワーカー定義マクロの実装
```

- **`#[event]`**: イベント型に`Event`トレイトを実装
- **`define_streams!`**: ストリーム設定を一元管理し、自動登録
- **`#[worker]`**: ワーカー定義のメタ情報を登録

---

## 📦 infra（JetStream 実装例）

```
infra/jetstream/
├─ src/lib.rs
│    ├─ connect(nats_url) → `JetStreamCtx`
│    └─ setup_all_streams(js) → ストリーム設定を列挙 & idempotent 作成
├─ src/js_publisher.rs      ← `EventPublisher for JsPublisher`
└─ src/js_subscriber.rs     ← `EventSubscriber for JsSubscriber`
```

- **JetStream** に特化した Publisher/Subscriber
- `define_streams!`マクロで定義されたストリームを自動プロビジョニング
- イベント型から自動的にdurable nameを生成

---

## 📦 app (workers)

```
app/
└─ src/
   ├─ main.rs
   └─ workers/
       ├─ epg_worker.rs
       └─ discord_worker.rs
```

- 各ワーカーは `WorkerBuilder::new(sub, pub, handler)`
  - `.with_middleware(...)`
  - `.durable_auto()`
  - `.run(shutdown_token)`
- **ミドルウェア**
  - ロギング／メトリクス／エラー分類(retry/ack)をプラグイン式に挟める
- **CLI**
  - `clap` を使用したコマンドライン引数の解析
  - サブコマンドで起動するワーカーを選択可能
  - 例: `kurec-app epg` でEPGワーカーを起動

---

## 🔄 エラーハンドリング & ミドルウェア

1. **ClassifyError** トレイト → `error_action(): ErrorAction`
   - `ErrorAction::Retry` → 再試行（nack）
   - `ErrorAction::Ignore` → 無視（ack）
2. **ミドルウェア層** でError分類 → retry/ack 決定
3. **DLQ** は専用パイプライン or 管理UIで手動再投入

---

## 🚀 開発フロー

1. **マクロ定義** (`shared-macros`) を確立
2. **ポート定義** (`shared-core/ports`) を整理
3. **ドメインユースケース** を書く
4. **infra 実装** を差し込む
5. **app ワーカー** を `WorkerBuilder` ベースで組み立て
6. **テスト**
   - trybuild (マクロ)
   - in‑process / testcontainers (JetStream E2E)
   - in‑memory 実装 (単体)

---
