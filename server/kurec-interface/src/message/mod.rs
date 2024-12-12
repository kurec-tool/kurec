use mirakc_client::models::{MirakurunProgram, MirakurunService};
use serde::{Deserialize, Serialize};

use crate::ProgramDocument;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EpgProgramsUpdatedMessage {
    pub tuner_url: String,
    pub service: MirakurunService,
    pub programs: Vec<MirakurunProgram>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProgramDocumentsUpdatedMessage {
    pub tuner_url: String,
    pub service: MirakurunService,
    pub programs: Vec<ProgramDocument>,
}
