use super::event::MirakcEvent;

pub struct MirakcEventMessage {
    pub tuner_url: String,
    pub event_data: MirakcEvent,
}
