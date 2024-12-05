# シーケンス図ドキュメント

## kurec-sse-epg

mirakc の `/events` SSE を受信して EPG データを取得し、NATS JetStream KV Store に保存/NATS JetStream Stream に publish する。

```mermaid
sequenceDiagram
participant mirakc
participant kurec-sse-epg

box nats
    participant kv-bucket-epg as KV store "kurec-epg" bucket
    participant js-epg as JetStream "kurec-epg" stream
end

mirakc ->>+ kurec-sse-epg: /events SSE

loop event受信
    alt epg.programs-updated(service_id)
        note right of kurec-sse-epg: EPG データが更新

        kurec-sse-epg ->>+ mirakc: /api/services/{service_id}/programs
        mirakc ->>- kurec-sse-epg: [program, program, ...]

        loop program in programs
            kurec-sse-epg ->>+ kv-bucket-epg: get("epg-programs.{program_id}")
            kv-bucket-epg ->>- kurec-sse-epg: {json: "{JSON-serialized epg}", ...}

            alt 受信したepgと保存されているepgが異なる
                kurec-sse-epg ->>+ kv-bucket-epg: put("epg-programs.{program_}", "{JSON-serialized epg}")
                note right of kv-bucket-epg: 削除はKV Storeの機能に任せて20～30日後に自動削除する
                kv-bucket-epg ->>- kurec-sse-epg: OK

                kurec-sse-epg -) js-epg: publish message("epg-programs.{service_id}", "{tuner_url}")
                note right of js-epg: この矢印は非同期呼び出しなので返答なし（以下同様）

            else 受信したepgと保存されているepgが同じ
                note right of kurec-sse-epg: 何もしない
                note right of kurec-sse-epg: 受信可能チューナーリストに追加するべき？
            end
        end
    else 他の場合
        note right of kurec-sse-epg: 何もしない
    end
end
deactivate kurec-sse-epg
```

## kurec-rule

NATS JetStream Stream からイベントを受信して何らかの処理をする

- EpgProgramsUpdatedMessage: EPG データが更新されたので Meilisearch に保存して検索し、録画予約の更新をする
- RuleUpdatedMessage: 録画ルールが更新されたので内部の録画ルールを更新する

```mermaid
sequenceDiagram
box nats
    participant kv-bucket-epg as KV store "kurec-epg" bucket
    participant kv-bucket-rule as KV store "rule" bucket
    participant js-epg as JetStream "kurec-epg" stream
end

participant mirakc
participant kurec-rule

participant meilisearch

activate js-epg
loop ずっと
    js-epg ->>+ kurec-rule: （consumerにメッセージが届く）

    alt EpgProgramsUpdatedMessage の場合
        note right of kurec-rule: EPG データが更新された
        kurec-rule ->>+ kv-bucket-epg: get("epg-programs.{service_id}")
        kv-bucket-epg ->>- kurec-rule: {json: "{JSON-serialized epg}", ...}

        note right of kurec-rule: Meilisearch保存Document用に変換
        kurec-rule ->>+ meilisearch: Add or update documents on index: epg-programs_{service_id}
        meilisearch ->>- kurec-rule: {task_uid: number}

        kurec-rule ->>+ meilisearch: wait on task(task_uid)
        meilisearch ->>- kurec-rule: {status: "completed"}

        kurec-rule ->>+ meilisearch: Search documents on index: epg-programs_{service_id} +filter: program_id = {program_id}

        alt 検索でヒット
            note right of kurec-rule: 録画予約する
            alt 既に予約済み
                kurec-rule ->>+ mirakc: GET /api/recording/schedule/{program_id}
                mirakc ->>- kurec-rule: OK
                note right of kurec-rule: 何もしない
            else まだ予約してない
                kurec-rule ->>+ mirakc: GET /api/recording/schedule/{program_id}
                kurec-rule ->>+ mirakc: POST /api/recording/schedule
                mirakc ->>- kurec-rule: OK
            end
        else 検索でヒットしない
            note right of kurec-rule: 録画予約解除する
            alt 既に予約済み
                kurec-rule ->>+ mirakc: GET /api/recording/schedule/{program_id}
                mirakc ->>- kurec-rule: OK
                kurec-rule ->>+ mirakc: DELETE /api/recording/schedule/{program_id}
                mirakc ->>- kurec-rule: OK
            else まだ予約してない
                note right of kurec-rule: 何もしない
            end
        end

    else RuleUpdatedMessage の場合
        note right of kurec-rule: 録画ルールが更新された
        kurec-rule ->>+ kv-bucket-rule: get("rules")
        note right of kv-bucket-rule: KV Storeのキー1個につき1つの録画ルールの方が良いかも？
        kv-bucket-rule ->>- kurec-rule: [rule, ...]
        note right of kurec-rule: 内部の録画ルールを更新する
    end

    kurec-rule ->>- js-epg: ack
end
deactivate js-epg
```

## kurec-record-retriever

mirakc の `/events` から SSE で録画ファイル作成完了イベント(recording.record-saved)を受信して、mirakc から録画ファイルを回収する

```mermaid
sequenceDiagram

participant mirakc
participant kurec-record-retriever
participant storage as S3互換ストレージ？

mirakc ->>+ kurec-record-retriever: /events SSE

loop event受信
    alt recording.record-saved
        note right of kurec-record-retriever: 録画ファイルの変更あり
        kurec-record-retriever ->>+ mirakc: GET /api/recording/records/{record_id}
        mirakc ->> kurec-record-retriever: OK: record
        alt recordingStatus == "finished"
            note right of kurec-record-retriever: 録画ファイル作成完了しているので取得する
            kurec-record-retriever ->>+ mirakc: GET /api/recording/records/{record_id}/stream
            mirakc ->>- kurec-record-retriever: OK: file stream

            kurec-record-retriever ->>+ storage: PUT /{record_id}.ts
            storage ->>- kurec-record-retriever: OK

            note right of kurec-record-retriever: KV StoreやMeilisearchにも保存する？

            kurec-record-retriever ->>+ mirakc: DELETE /api/recording/records/{record_id}
                note right of mirakc: ディスクから削除する
            mirakc ->>- kurec-record-retriever: OK
        else 他の場合
            note right of kurec-record-retriever: 何もしない
        end
    else 他の場合
        note right of kurec-record-retriever: 何もしない
    end
end

deactivate kurec-record-retriever
```

