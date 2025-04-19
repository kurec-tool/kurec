use crate::event_metadata::Event;

#[async_trait::async_trait]
pub trait EventPublisher<E: Event>: Send + Sync + 'static {
    async fn publish(&self, event: E) -> anyhow::Result<()>;
}
