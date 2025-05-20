use crate::{
    error,
    models::{
        Creator, CreatorReplayInfo, CreatorSaveInfo, CreatorStateInfo, Instance, InstanceWork,
        Replay, Save, State,
    },
};
use meilisearch_sdk::client::Client as Meili;
use meilisearch_sdk::task_info::TaskInfo;
use sqlx::postgres::PgConnection;

const CHUNK_SIZE: usize = 10000;

pub trait SearchIndexer {
    type IndexOut: std::fmt::Debug;
    fn upsert_instance(
        &self,
        conn: &mut PgConnection,
        instance: &Instance,
    ) -> impl std::future::Future<Output = Result<Self::IndexOut, error::SearchIndex>> + Send;
    fn upsert_save(
        &self,
        conn: &mut PgConnection,
        save: &Save,
    ) -> impl std::future::Future<Output = Result<Self::IndexOut, error::SearchIndex>> + Send;
    fn upsert_state(
        &self,
        conn: &mut PgConnection,
        state: &State,
    ) -> impl std::future::Future<Output = Result<Self::IndexOut, error::SearchIndex>> + Send;
    fn upsert_replay(
        &self,
        conn: &mut PgConnection,
        replay: &Replay,
    ) -> impl std::future::Future<Output = Result<Self::IndexOut, error::SearchIndex>> + Send;
    fn upsert_creator(
        &self,
        conn: &mut PgConnection,
        creator: &Creator,
    ) -> impl std::future::Future<Output = Result<Self::IndexOut, error::SearchIndex>> + Send;
    fn reindex(
        &self,
        conn: &mut PgConnection,
    ) -> impl std::future::Future<Output = Vec<Result<Self::IndexOut, error::SearchIndex>>> + Send;
}

#[derive(Debug, Clone)]
pub struct MeiliIndexer {
    meili: Meili,
}

impl MeiliIndexer {
    pub fn new(url: &str, api_key: &str) -> Result<Self, crate::error::SearchIndex> {
        Ok(Self {
            meili: Meili::new(url, Some(api_key))?,
        })
    }
}

impl SearchIndexer for MeiliIndexer {
    type IndexOut = TaskInfo;
    async fn upsert_instance(
        &self,
        conn: &mut PgConnection,
        instance: &Instance,
    ) -> Result<Self::IndexOut, error::SearchIndex> {
        let iw = InstanceWork::get_for_instance(conn, instance.instance_id).await?;
        self.meili
            .index("instance")
            .add_or_update(&[iw], Some("instance_id"))
            .await
            .map_err(crate::error::SearchIndex::from)
    }

    async fn upsert_save(
        &self,
        conn: &mut PgConnection,
        save: &Save,
    ) -> Result<Self::IndexOut, error::SearchIndex> {
        let infos = CreatorSaveInfo::get_for_save(conn, save).await?;
        self.meili
            .index("save")
            .add_or_update(&infos, Some("save_id"))
            .await
            .map_err(crate::error::SearchIndex::from)
    }

    async fn upsert_state(
        &self,
        conn: &mut PgConnection,
        state: &State,
    ) -> Result<Self::IndexOut, error::SearchIndex> {
        let info = CreatorStateInfo::get_for_state(conn, state).await?;
        self.meili
            .index("state")
            .add_or_update(&[info], Some("state_id"))
            .await
            .map_err(crate::error::SearchIndex::from)
    }

    async fn upsert_replay(
        &self,
        conn: &mut PgConnection,
        replay: &Replay,
    ) -> Result<Self::IndexOut, error::SearchIndex> {
        let info = CreatorReplayInfo::get_for_replay(conn, replay).await?;
        self.meili
            .index("replay")
            .add_or_update(&[info], Some("replay_id"))
            .await
            .map_err(crate::error::SearchIndex::from)
    }

    async fn upsert_creator(
        &self,
        _conn: &mut PgConnection,
        creator: &Creator,
    ) -> Result<Self::IndexOut, error::SearchIndex> {
        self.meili
            .index("save")
            .add_or_update(&[creator], Some("creator_id"))
            .await
            .map_err(crate::error::SearchIndex::from)
    }
    async fn reindex(
        &self,
        conn: &mut PgConnection,
    ) -> Vec<Result<Self::IndexOut, error::SearchIndex>> {
        use futures::StreamExt;
        let mut instances: Vec<_> = InstanceWork::get_stream(conn)
            .chunks(CHUNK_SIZE)
            .then(async |chunk| {
                let idx = self.meili.index("instance");
                idx.add_or_update(&(chunk), Some("instance_id"))
                    .await
                    .map_err(crate::error::SearchIndex::from)
            })
            .collect()
            .await;
        let saves: Vec<_> = CreatorSaveInfo::get_stream(conn)
            .chunks(CHUNK_SIZE)
            .then(async |chunk| {
                let idx = self.meili.index("save");
                idx.add_or_update(&(chunk), Some("save_id"))
                    .await
                    .map_err(crate::error::SearchIndex::from)
            })
            .collect()
            .await;
        let states: Vec<_> = CreatorStateInfo::get_stream(conn)
            .chunks(CHUNK_SIZE)
            .then(async |chunk| {
                let idx = self.meili.index("state");
                idx.add_or_update(&(chunk), Some("state_id"))
                    .await
                    .map_err(crate::error::SearchIndex::from)
            })
            .collect()
            .await;
        let replays: Vec<_> = CreatorReplayInfo::get_stream(conn)
            .chunks(CHUNK_SIZE)
            .then(async |chunk| {
                let idx = self.meili.index("replay");
                idx.add_or_update(&(chunk), Some("replay_id"))
                    .await
                    .map_err(crate::error::SearchIndex::from)
            })
            .collect()
            .await;
        let creators: Vec<_> = Creator::get_stream(conn)
            .chunks(CHUNK_SIZE)
            .then(async |chunk| {
                let idx = self.meili.index("creator");
                idx.add_or_update(&(chunk), Some("creator_id"))
                    .await
                    .map_err(crate::error::SearchIndex::from)
            })
            .collect()
            .await;
        instances.extend(saves);
        instances.extend(states);
        instances.extend(replays);
        instances.extend(creators);
        instances
    }
}

#[derive(Debug, Clone)]
pub struct MeiliSearch {
    meili: Meili<meilisearch_sdk::reqwest::ReqwestClient>,
}
impl MeiliSearch {
    pub fn new(url: &str, api_key: &str) -> Result<Self, crate::error::Search> {
        Ok(Self {
            meili: Meili::new(url, Some(api_key))?,
        })
    }
    pub fn instances(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("instance")
    }
    pub fn saves(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("save")
    }
    pub fn states(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("state")
    }
    pub fn replays(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("replay")
    }
    pub fn creators(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("creator")
    }
}
