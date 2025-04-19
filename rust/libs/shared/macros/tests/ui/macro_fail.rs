use shared_macros::event;

#[event(subject = "foo.bar")] // ← stream が無いのでコンパイル失敗
pub struct Bad;
