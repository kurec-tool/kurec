use anyhow::Result;
use meilisearch_sdk::search::SearchQuery;
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Document {
    id: u64,
    title: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let meilisearch_url = "http://meilisearch:7700";
    let meilisearch_api_key: Option<String> = None;

    let client =
        meilisearch_sdk::client::Client::new(meilisearch_url, meilisearch_api_key.clone())?;

    let index = client.index("test-index");
    let documents = vec![
        Document {
            id: 1,
            title: "Document 1".to_string(),
            description: "Description 1".to_string(),
        },
        Document {
            id: 2,
            title: "Document 2".to_string(),
            description: "Description 2".to_string(),
        },
    ];
    let task = index.add_documents::<Document>(&documents, None).await?;
    task.wait_for_completion(&client, None, None).await?;

    let sq1 = SearchQuery::new(&index).with_query("Document").build();
    let sq2 = SearchQuery::new(&index).with_query("Description").build();
    let response = client
        .multi_search()
        .with_search_query(sq1)
        .with_search_query(sq2)
        .execute::<Document>()
        .await?;
    dbg!(&response);
    Ok(())
}
