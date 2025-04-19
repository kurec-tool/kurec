use shared_core::event_metadata::HasStreamDef as _;
use shared_macros::event;

#[event(stream = "test", subject = "subj")]
pub struct MyEvt;

#[test]
fn has_stream_def_trait() {
    assert_eq!(MyEvt::stream_name(), "test");
    assert_eq!(MyEvt::stream_subject(), "subj");
}
