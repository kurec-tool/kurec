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

## ⚙️ shared-core（ports & metadata）

```
shared/core/
├─ src/
│  ├── ports/
│  │    ├─ event.rs            ← `pub trait Event` マーカー
│  │    ├─ event_publisher.rs  ← `pub trait EventPublisher`
│  │    └─ event_subscriber.rs ← `pub trait EventSubscriber`, `AckHandle`
│  └── event_metadata.rs       ← `StreamDef` / `HasStreamDef` / `parse_duration`
```

- **`Event`**: 全ての `#[event]` 型が実装するマーカー
- **`StreamDef`／`HasStreamDef`**: subject/stream 名を型から取得
- **`EventPublisher`／`EventSubscriber`**: 入出力の抽象ポート

---

## 🛠 shared-macros（コード生成）

```
shared/macros/  (proc‑macro = true)
└─ src/lib.rs
   ├─ #[event(stream=…, subject=…)] → StreamDef 登録＋HasStreamDef 実装
   └─ #[worker(...)]                 → WorkerDef 登録＋属性パース
```

- イベント定義・ワーカー定義のメタ情報を `inventory` に流し込む
- 実行時登録／起動ロジックと疎結合に

---

## 📦 infra（JetStream 実装例）

```
infra/jetstream/
├─ src/lib.rs
│    ├─ connect(nats_url) → `JetStreamCtx`
│    └─ setup_all_streams(js) → StreamDef 列挙 & idempotent 作成
├─ src/js_publisher.rs      ← `EventPublisher for JsPublisher`
└─ src/js_subscriber.rs     ← `EventSubscriber for JsSubscriber`
```

- **JetStream** に特化した Publisher/Subscriber
- マクロで定義されたストリームを自動プロビジョニング

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

---

## 🔄 エラーハンドリング & ミドルウェア

1. **ClassifyError** トレイト → `should_retry: bool`
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

