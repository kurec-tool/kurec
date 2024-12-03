use actix::prelude::*;
use kurec::{
    message::ping_pong::Ping,
    model::{
        meilisearch::program::ProgramDocument,
        mirakurun::{program::Programs, service::Service},
    },
};
use tracing::{error, info};

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateEpgProgramsMessage {
    pub service_id: u64,
}

async fn get_programs(tuner_host: &str, service_id: u64) -> Result<Programs, reqwest::Error> {
    let url = format!(
        "http://{}:40772/api/services/{}/programs",
        tuner_host, service_id
    );
    let resp = reqwest::get(&url).await.unwrap();
    let programs = resp.json::<Programs>().await.unwrap();
    Ok(programs)
}

async fn get_service(tuner_host: &str, service_id: u64) -> Result<Service, reqwest::Error> {
    let url = format!("http://{}:40772/api/services/{}", tuner_host, service_id);
    let resp = reqwest::get(&url).await.unwrap();
    let service = resp.json::<Service>().await.unwrap();
    Ok(service)
}

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
    dbg!(&task);
    task.wait_for_completion(client, None, None).await.unwrap();
    let index = client.index(index_uid);
    dbg!(&index);
    dbg!(index.get_searchable_attributes().await.unwrap());
    Ok(index)
}
async fn update_programs(
    index: &meilisearch_sdk::indexes::Index,
    programs: Programs,
    service: Service,
) -> Result<(), meilisearch_sdk::errors::Error> {
    let documents = programs
        .iter()
        .map(|p| ProgramDocument::from_mirakurun(p, &service))
        .filter(|p| p.is_some())
        .collect::<Vec<_>>();
    index.add_or_replace(&documents, None).await?;
    Ok(())
}

pub struct ProgramRegister {
    tuner_host: String,
    meilisearch_host: String,
    meilisearch_api_key: Option<String>,
    meilisearch_client: Option<meilisearch_sdk::client::Client>,
}

impl ProgramRegister {
    pub fn new(
        tuner_host: String,
        meilisearch_host: String,
        meilisearch_api_key: Option<String>,
    ) -> Self {
        ProgramRegister {
            tuner_host,
            meilisearch_host,
            meilisearch_client: None,
            meilisearch_api_key,
        }
    }
}

impl Actor for ProgramRegister {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("ProgramRegister started");
        dbg!(&self.meilisearch_host, &self.meilisearch_api_key);
        let meilisearch_url = format!("http://{}:7700", self.meilisearch_host);
        match meilisearch_sdk::client::Client::new(
            meilisearch_url,
            self.meilisearch_api_key.clone(),
        ) {
            Ok(client) => {
                self.meilisearch_client = Some(client);
            }
            Err(e) => {
                error!("meilisearch connect error: {:?}", e);
                ctx.stop();
            }
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("ProgramRegister stopped");
    }
}

impl Supervised for ProgramRegister {
    fn restarting(&mut self, _ctx: &mut Self::Context) {
        info!("ProgramRegister restarting");
    }
}

impl Handler<UpdateEpgProgramsMessage> for ProgramRegister {
    type Result = ();

    fn handle(&mut self, msg: UpdateEpgProgramsMessage, ctx: &mut Self::Context) -> Self::Result {
        info!("UpdateEpgProgramsMessage: {:?}", msg.service_id);
        let tuner_host = self.tuner_host.clone();
        let meilisearch_client = self.meilisearch_client.clone();
        async move {
            let programs = get_programs(&tuner_host, msg.service_id).await.unwrap();
            let service = get_service(&tuner_host, msg.service_id).await.unwrap();
            let meilisearch_client = meilisearch_client.unwrap();
            let index = get_meilisearch_index(&meilisearch_client, msg.service_id)
                .await
                .unwrap();
            update_programs(&index, programs, service).await.unwrap();
        }
        .into_actor(self)
        .wait(ctx);
    }
}

impl Handler<Ping> for ProgramRegister {
    type Result = bool;

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        true
    }
}
