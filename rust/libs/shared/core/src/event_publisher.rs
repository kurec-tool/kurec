use crate::event_metadata::Event;

#[async_trait::async_trait]
pub trait EventPublisher<E: Event> {
    async fn publish(&self, event: E) -> anyhow::Result<()>;
}
