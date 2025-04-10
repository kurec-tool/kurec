use kurec_adapter::NatsAdapter;

#[derive(Clone, Debug)]
pub struct InitializeDomain {
    nats_adapter: NatsAdapter,
}

impl InitializeDomain {
    pub fn new(nats_adapter: NatsAdapter) -> Self {
        Self { nats_adapter }
    }

    pub async fn initialize(&self) -> Result<(), anyhow::Error> {
        // ここで初期化処理を行う
        // 例えば、Meilisearchのインデックス作成や、NATSのストリーム作成など
        self.nats_adapter.initialize().await?;
        Ok(())
    }
}
