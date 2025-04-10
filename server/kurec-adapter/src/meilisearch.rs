use kurec_interface::{KurecConfig, MeilisearchIndexConfig};
use meilisearch_sdk::{
    client::{Client, SwapIndexes},
    documents::DocumentsQuery,
};
use serde::{de::DeserializeOwned, Serialize};
use tracing::debug;

pub enum MeilisearchIndex {
    Epg,
}

impl MeilisearchIndex {
    fn as_str(&self) -> &str {
        match self {
            MeilisearchIndex::Epg => "epg",
        }
    }
    fn get_index_config(&self, config: &KurecConfig) -> MeilisearchIndexConfig {
        match self {
            MeilisearchIndex::Epg => config.meilisearch.epg.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MeilisearchAdapter {
    config: KurecConfig,
}

impl MeilisearchAdapter {
    pub async fn new_async(config: KurecConfig) -> Result<Self, anyhow::Error> {
        let me = Self { config };
        let client = me.get_client()?;
        let indexes = [MeilisearchIndex::Epg];
        for index in indexes.iter() {
            let index_name = me.get_prefixed_index_name(&MeilisearchIndex::Epg);
            let config = index.get_index_config(&me.config);
            match client.get_index(&index_name).await {
                Ok(_) => {}
                Err(_) => {
                    let task = client
                        .create_index(&index_name, Some("program_id"))
                        .await
                        .unwrap(); // unwrapにすることでindex作れないようなエラーなら落とす
                    task.wait_for_completion(&client, None, None).await.unwrap();
                    let index = client.get_index(&index_name).await.unwrap();
                    index
                        .set_searchable_attributes(config.searchable_attributes.clone())
                        .await
                        .unwrap();
                    index
                        .set_displayed_attributes(config.displayed_attributes.clone())
                        .await
                        .unwrap();
                    index
                        .set_filterable_attributes(config.filterable_attributes.clone())
                        .await
                        .unwrap();
                    index
                        .set_sortable_attributes(config.sortable_attributes.clone())
                        .await
                        .unwrap();
                }
            }
        }
        Ok(me)
    }

    fn get_client(&self) -> Result<Client, anyhow::Error> {
        Client::new(
            &self.config.meilisearch.url,
            self.config.meilisearch.api_key.clone(),
        )
        .map_err(anyhow::Error::new)
    }

    fn get_prefixed_index_name(&self, index: &MeilisearchIndex) -> String {
        format!("{}-{}", self.config.prefix, index.as_str())
    }

    async fn get_index(
        &self,
        index: &MeilisearchIndex,
    ) -> Result<meilisearch_sdk::indexes::Index, anyhow::Error> {
        let client = self.get_client()?;
        client
            .get_index(&self.get_prefixed_index_name(index))
            .await
            .map_err(anyhow::Error::new)
    }

    pub async fn update_documents<T, F>(
        &self,
        index_name: &MeilisearchIndex,
        delete_filter: F,
        update_documents: &[T],
        primary_key: Option<&str>,
    ) -> Result<(), anyhow::Error>
    where
        T: DeserializeOwned + Serialize + 'static + Send + Sync + Clone,
        F: Fn(&T) -> bool,
    {
        debug!("get_index");
        let index = self.get_index(index_name).await.unwrap();
        debug!("creating client");
        let client = self.get_client()?;
        debug!("creating tmp index");
        let tmp_index_name = format!(
            "{}-{}-tmp",
            self.get_prefixed_index_name(index_name),
            uuid::Uuid::new_v4()
        );
        let task_info = client
            .create_index(&tmp_index_name, primary_key)
            .await
            .unwrap();
        task_info
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();
        let config = index_name.get_index_config(&self.config);
        let tmp_index = client.get_index(&tmp_index_name).await.unwrap();
        tmp_index
            .set_searchable_attributes(config.searchable_attributes.clone())
            .await
            .unwrap();
        tmp_index
            .set_displayed_attributes(config.displayed_attributes)
            .await
            .unwrap();
        tmp_index
            .set_filterable_attributes(config.filterable_attributes)
            .await
            .unwrap();
        tmp_index
            .set_sortable_attributes(config.sortable_attributes)
            .await
            .unwrap();

        let resp = index
            .get_documents_with::<T>(DocumentsQuery::new(&index).with_limit(1))
            .await
            .unwrap();
        let resp = index
            .get_documents_with::<T>(DocumentsQuery::new(&index).with_limit(resp.total as usize))
            .await
            .unwrap();
        debug!(
            "get_documents returns {} documents. limits:{} offset:{}",
            resp.total, resp.limit, resp.offset
        );
        let old_documents = resp
            .results
            .iter()
            .filter(|doc| !delete_filter(doc))
            .cloned()
            .collect::<Vec<T>>();
        debug!(
            "old_documents.len: {} new documents len: {} total: {}",
            old_documents.len(),
            update_documents.len(),
            old_documents.len() + update_documents.len()
        );
        tmp_index
            .add_documents(&old_documents, primary_key)
            .await
            .unwrap();
        tmp_index
            .add_documents(update_documents, primary_key)
            .await
            .unwrap();
        client
            .swap_indexes([&SwapIndexes {
                indexes: (index.uid.clone(), tmp_index.uid.clone()),
            }])
            .await
            .unwrap();
        let task = client.delete_index(tmp_index.uid).await.unwrap();
        // TODO: interval, timeoutの設定ファイル化
        task.wait_for_completion(&client, None, Some(std::time::Duration::from_secs(60)))
            .await
            .unwrap();
        Ok(())
    }
}
