use kurec_adapter::{KvsType, NatsAdapter, OgpAdapter, StreamType};
use kurec_interface::{KurecConfig, OgpRequestMessage};
use tracing::debug;

pub struct OgpDomain {
    config: KurecConfig,
    nats_adapter: NatsAdapter,
    ogp_adapter: OgpAdapter,
}

impl OgpDomain {
    pub fn new(config: KurecConfig, nats_adapter: NatsAdapter, ogp_adapter: OgpAdapter) -> Self {
        Self {
            config,
            nats_adapter,
            ogp_adapter,
        }
    }

    pub async fn collect_ogp_stream(&self) -> Result<(), anyhow::Error> {
        let width = self.config.ogp_width;
        let f = |ev: OgpRequestMessage| async move {
            debug!("event: {:?}", ev);
            if self
                .nats_adapter
                .kv_exists_key(KvsType::Ogp, &ev.hash)
                .await
                .unwrap()
            {
                return Ok(());
            }
            let ogp = self
                .ogp_adapter
                .get_resized_ogp_webps_from_url(&ev.url, width)
                .await;
            if let Ok(Some(ogp)) = ogp {
                self.nats_adapter
                    .kv_put_bytes(KvsType::Ogp, &ev.hash, &*ogp)
                    .await
                    .unwrap();
            }
            self.nats_adapter
                .kv_put_str(KvsType::UrlHash, &ev.hash, &ev.url)
                .await
                .unwrap();
            Ok(())
        };

        self.nats_adapter
            .stream_sink_async(StreamType::OgpRequest, "collector", f)
            .await
            .unwrap();
        Err(anyhow::anyhow!("unreachable.unwrap()"))
    }
}
