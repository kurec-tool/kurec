use kurec::model::{
    meilisearch::program::ProgramDocument,
    mirakurun::{program::Programs, service::Service},
};
use meilisearch_sdk::search::{MatchingStrategies, SearchQuery};
use serde::de::{self, Deserializer as SerdeDeserializer, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::debug;

async fn get_meilisearch_index(
    client: &meilisearch_sdk::client::Client,
    service_id: u64,
) -> Result<meilisearch_sdk::indexes::Index, meilisearch_sdk::errors::Error> {
    let index_uid = format!("epg-programs_{}", service_id);
    let index = client.index(index_uid.clone());
    let filterable = ProgramDocument::get_filterable_attributes();
    let searchable = ProgramDocument::get_searchable_attributes();
    let sortable = ProgramDocument::get_sortable_attributes();

    index.set_filterable_attributes(&filterable).await.unwrap();
    index.set_sortable_attributes(&sortable).await.unwrap();
    let task = index.set_searchable_attributes(&searchable).await.unwrap();
    task.wait_for_completion(client, None, None).await.unwrap();
    let index = client.index(index_uid);
    Ok(index)
}

pub async fn update_programs(
    meilisearch_url: &str,
    meilisearch_api_key: &Option<String>,
    programs: &Programs,
    service: &Service,
) -> Result<(), meilisearch_sdk::errors::Error> {
    let client =
        meilisearch_sdk::client::Client::new(meilisearch_url, meilisearch_api_key.clone())?;
    let service_id = service.id;
    let index = get_meilisearch_index(&client, service_id).await?;
    let documents = programs
        .iter()
        .map(|p| ProgramDocument::from_mirakurun(p, service))
        .filter(|p| p.is_some())
        .collect::<Vec<_>>();
    let task_info = index.add_or_replace(&documents, None).await?;
    task_info.wait_for_completion(&client, None, None).await?;
    Ok(())
}

fn deserialize_matching_strategies<'de, D>(
    deserializer: D,
) -> Result<Option<MatchingStrategies>, D::Error>
where
    D: SerdeDeserializer<'de>,
{
    struct MatchingStrategiesVisitor;

    impl<'de> Visitor<'de> for MatchingStrategiesVisitor {
        type Value = Option<MatchingStrategies>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an optional string representing a MatchingStrategies variant")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: SerdeDeserializer<'de>,
        {
            deserializer.deserialize_str(self)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                "all" => Ok(Some(MatchingStrategies::ALL)),
                "last" => Ok(Some(MatchingStrategies::LAST)),
                "frequency" => Ok(Some(MatchingStrategies::FREQUENCY)),
                _ => Err(de::Error::unknown_variant(
                    value,
                    &["all", "last", "frequency"],
                )),
            }
        }
    }

    deserializer.deserialize_option(MatchingStrategiesVisitor)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MeilisearchQuery {
    #[serde(rename = "q")]
    query: Option<String>,
    filter: Option<String>,
    #[serde(deserialize_with = "deserialize_matching_strategies")]
    matching_strategy: Option<MatchingStrategies>,
}

const TEST_QUERIES_JSON: &str = r#"
[
    {
        "q": "\"聖☆おにいさん\"",
        "matchingStrategy": "all"
    },
    {
        "q": "\"ダンダダン\"",
        "filter": "チャンネル名 = \"NHK総合1・東京\"",
        "matchingStrategy": "all"
    }
]"#;

pub async fn search_program_ids(
    meilisearch_url: &str,
    meilisearch_api_key: &Option<String>,
    service: &Service,
) -> Result<Vec<u64>, anyhow::Error> {
    let client =
        meilisearch_sdk::client::Client::new(meilisearch_url, meilisearch_api_key.clone())?;
    let index = get_meilisearch_index(&client, service.id).await?;

    let queries: Vec<MeilisearchQuery> = serde_json::from_str(TEST_QUERIES_JSON)?;
    let mut multi_search = client.multi_search();

    for query in &queries {
        let mut search_query = SearchQuery::new(&index);
        if let Some(query) = &query.query {
            search_query.with_query(query);
        }
        if let Some(filter) = &query.filter {
            search_query.with_filter(filter);
        }
        if let Some(matching_strategy) = &query.matching_strategy {
            search_query.with_matching_strategy(matching_strategy.clone());
        }
        multi_search.with_search_query(search_query.build());
    }
    let search_result = multi_search.execute::<ProgramDocument>().await?;
    let program_ids = search_result
        .results
        .iter()
        .flat_map(|r| r.hits.iter().map(|h| h.result.id))
        .collect::<Vec<_>>();
    debug!("program_ids: {:?}", program_ids);
    Ok(program_ids)
}
