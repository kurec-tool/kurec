use crate::model::{
    meilisearch::{program::ProgramDocument, MeilisearchQuery},
    mirakurun::{
        program::{Program, Programs},
        service::Service,
    },
};
use meilisearch_sdk::search::SearchQuery;
use serde::Deserialize;
use tracing::{debug, warn};

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

pub async fn is_program_matching_rule(
    meilisearch_url: &str,
    meilisearch_api_key: &Option<String>,
    service: &Service,
    program: &Program,
    query: &MeilisearchQuery,
) -> Result<bool, meilisearch_sdk::errors::Error> {
    let client =
        meilisearch_sdk::client::Client::new(meilisearch_url, meilisearch_api_key.clone())?;

    let tmp_index_uid = format!("tmp_{}", uuid::Uuid::now_v7());
    let index = client.index(tmp_index_uid.clone());
    let filterable = ProgramDocument::get_filterable_attributes();
    let searchable = ProgramDocument::get_searchable_attributes();
    let sortable = ProgramDocument::get_sortable_attributes();

    index.set_filterable_attributes(&filterable).await?;
    index.set_sortable_attributes(&sortable).await?;
    index.set_searchable_attributes(&searchable).await?;

    let task = index
        .add_documents(&[ProgramDocument::from_mirakurun(program, service)], None)
        .await?;
    task.wait_for_completion(&client, None, None).await?;

    let mut search = index.search();
    if let Some(query) = &query.query {
        search.with_query(query);
    }
    if let Some(filter) = &query.filter {
        search.with_filter(filter);
    }
    if let Some(matching_strategy) = &query.matching_strategy {
        search.with_matching_strategy(matching_strategy.clone());
    }
    let search_result = search.execute::<ProgramDocument>().await?;

    let result = search_result.hits.is_empty();
    match client.delete_index(tmp_index_uid).await {
        Ok(_) => {}
        Err(e) => {
            warn!("Error occurs when deleting index: {:?}", e);
        }
    };

    Ok(result)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    program_id: u64,
}

pub async fn list_rule_matched_program_ids(
    meilisearch_url: &str,
    meilisearch_api_key: &Option<String>,
    service: &Service,
    programs: &Programs,
    query: &MeilisearchQuery,
) -> Result<Vec<u64>, meilisearch_sdk::errors::Error> {
    debug!(
        "appling rule programs count: {} query: {:?}",
        programs.len(),
        query
    );
    let client =
        meilisearch_sdk::client::Client::new(meilisearch_url, meilisearch_api_key.clone())?;

    let tmp_index_uid = format!("tmp_{}", uuid::Uuid::now_v7());
    let index = client.index(tmp_index_uid.clone());
    let filterable = ProgramDocument::get_filterable_attributes();
    let searchable = ProgramDocument::get_searchable_attributes();
    let sortable = ProgramDocument::get_sortable_attributes();

    index.set_filterable_attributes(&filterable).await?;
    index.set_sortable_attributes(&sortable).await?;
    index.set_searchable_attributes(&searchable).await?;

    let documents: Vec<ProgramDocument> = programs
        .iter()
        .filter_map(|p| ProgramDocument::from_mirakurun(p, service))
        .collect();

    let task = index.add_documents(&documents, None).await?;
    debug!("start waiting for completion...");
    task.wait_for_completion(&client, None, None).await?;
    debug!("done.");

    let mut search = index.search();
    if let Some(query) = &query.query {
        search.with_query(query);
    }
    if let Some(filter) = &query.filter {
        search.with_filter(filter);
    }
    if let Some(matching_strategy) = &query.matching_strategy {
        search.with_matching_strategy(matching_strategy.clone());
    }
    search.with_limit(programs.len());
    search.with_attributes_to_retrieve(meilisearch_sdk::search::Selectors::Some(&["programId"]));
    debug!("start searching...");
    let search_result = search.execute::<SearchResult>().await?;
    debug!("search done. hits count: {}", search_result.hits.len());
    // dbg!(&search_result);

    match client.delete_index(tmp_index_uid).await {
        Ok(_) => {}
        Err(e) => {
            warn!("Error occurs when deleting index: {:?}", e);
        }
    };

    if !search_result.hits.is_empty() {
        dbg!(&search_result);
    }

    Ok(search_result
        .hits
        .iter()
        .map(|h| h.result.program_id)
        .collect())
}
