use shared_core::event_metadata;
use shared_macros::event;

// ここでダミーのイベント型を定義し、inventory に submit させる
#[event(
    stream = "test-stream",
    subject = "test.subject",
    max_deliver = 5,
    ack_wait = "1m",
    max_age = "1h"
)]
struct _TestStream; // This struct is used in the test below

#[test]
fn streamdef_registered() {
    // 1つだけ登録されているはず
    let defs: Vec<_> = shared_core::event_metadata::inventory::iter::<event_metadata::StreamDef>
        .into_iter()
        .collect();
    assert_eq!(defs.len(), 1);
    let def = defs[0];
    assert_eq!(def.name, "test-stream");
    assert_eq!(def.subjects, &["test.subject"]);
    assert_eq!(def.default_config.max_deliver, Some(5)); // ← マクロ指定に合わせる
}
