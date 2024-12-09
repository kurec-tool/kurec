use crate::model::mirakurun::{program::Programs, service::Service};

use crate::domain::meili_rule::apply_meili_rule;
use tracing::error;

pub async fn apply_rule(programs: &Programs, service: &Service) -> Result<usize, anyhow::Error> {
    match apply_meili_rule(programs, service).await {
        Ok(num_applied) => Ok(num_applied),
        Err(e) => {
            error!("Meilisearch rule error: {:?}", e);
            Err(e)
        }
    }
}
