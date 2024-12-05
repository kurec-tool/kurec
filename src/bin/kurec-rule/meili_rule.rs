use kurec::model::mirakurun::{program::Programs, service::Service};
use tracing::debug;

use crate::{
    meilisearch::{search_program_ids, update_programs},
    mirakc::schedule_program,
};

pub async fn apply_meili_rule(
    programs: &Programs,
    service: &Service,
) -> Result<usize, anyhow::Error> {
    let meilisearch_url = "http://meilisearch:7700";
    let meilisearch_api_key: Option<String> = None;
    let tuner_url = "http://tuner:40772";

    update_programs(meilisearch_url, &meilisearch_api_key, programs, service)
        .await
        .unwrap();

    let program_ids = search_program_ids(meilisearch_url, &meilisearch_api_key, service)
        .await
        .unwrap();
    debug!("program_ids: {:?}", program_ids);

    for program_id in &program_ids {
        schedule_program(tuner_url, program_id).await?;
    }

    Ok(program_ids.len())
}
