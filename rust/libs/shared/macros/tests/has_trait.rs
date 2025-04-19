use serde::{Deserialize, Serialize};
use shared_core::event_metadata::Event;
use shared_macros::event;

#[event(stream = "test")]
#[derive(Serialize, Deserialize)]
pub struct MyEvt;

#[test]
fn event_trait() {
    assert_eq!(<MyEvt as Event>::stream_name(), "test");
    assert_eq!(<MyEvt as Event>::event_name(), "my_evt");
    assert_eq!(<MyEvt as Event>::stream_subject(), "test.my_evt");
}
