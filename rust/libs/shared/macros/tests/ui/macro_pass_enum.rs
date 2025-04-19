use shared_macros::event; // inventory 参照でリンク

//#[event(stream = "test-stream", subject = "test.subject")]
#[event(
    stream = "test-stream",
    subject = "test.subject",
    max_deliver = 5,
    ack_wait = "1m",
    max_age = "1h"
)]
pub enum TestEvt {
    Evt1,
}

fn main() {
    // コンパイルさえ通れば OK
}
