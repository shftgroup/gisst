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
    /// # Errors
    /// If the address is invalid, creating the client will fail
    pub fn new(url: &str, api_key: &str) -> Result<Self, crate::error::SearchIndex> {
        Ok(Self {
            meili: Meili::new(url, Some(api_key))?,
        })
    }
    /// # Errors
    /// If the configured search URL is no good or another meili API problem occurs
    pub async fn init_indices(&self) -> Result<(), crate::error::SearchIndex> {
        let mut instances = self.meili.index("instance");
        instances.set_primary_key("instance_id").await?;
        instances.set_filterable_attributes(["work_platform"]).await?;
        instances.set_sortable_attributes(["work_name", "work_version", "work_platform"]).await?;
        let mut states = self.meili.index("state");
        states.set_primary_key("state_id").await?;
        states.set_filterable_attributes(["work_platform", "creator_id", "instance_id", "work_name"]).await?;
        states.set_sortable_attributes(["work_name", "work_version", "work_platform", "creator_username", "creator_full_name", "state_name", "created_on"]).await?;
        let mut saves = self.meili.index("save");
        saves.set_primary_key("save_id").await?;
        saves.set_filterable_attributes(["work_platform", "creator_id", "instance_id", "work_name"]).await?;
        saves.set_sortable_attributes(["work_name", "work_version", "work_platform", "creator_username", "creator_full_name", "save_short_desc", "created_on"]).await?;
        let mut replays = self.meili.index("replay");
        replays.set_primary_key("replay_id").await?;
        replays.set_filterable_attributes(["work_platform", "creator_id", "instance_id", "work_name"]).await?;
        replays.set_sortable_attributes(["work_name", "work_version", "work_platform", "creator_username", "creator_full_name", "replay_name", "created_on"]).await?;
        let mut creators = self.meili.index("creator");
        creators.set_primary_key("creator_id").await?;
        Ok(())
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
        if let Err(e) = self.init_indices().await {
            return vec![Err(e)];
        }
        let mut outputs = Vec::with_capacity(128);
        let instances: Vec<_> = InstanceWork::get_stream(conn)
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
        outputs.extend(instances);
        outputs.extend(saves);
        outputs.extend(states);
        outputs.extend(replays);
        outputs.extend(creators);
        outputs
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MeiliSearch {
    url: String,
    external_url: String,
    key: String,
    meili: Meili<meilisearch_sdk::reqwest::ReqwestClient>,
}
impl MeiliSearch {
    /// # Errors
    /// If the address is invalid, creating the client will fail
    pub fn new(
        url: &str,
        external_url: &str,
        search_key: &str,
    ) -> Result<Self, crate::error::Search> {
        Ok(Self {
            url: url.to_string(),
            external_url: external_url.to_string(),
            key: search_key.to_string(),
            meili: Meili::new(url, Some(search_key))?,
        })
    }
    #[must_use]
    pub fn frontend_data(&self) -> (&str, &str) {
        (&self.external_url, &self.key)
    }
    #[must_use]
    pub fn instances(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("instance")
    }
    #[must_use]
    pub fn saves(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("save")
    }
    #[must_use]
    pub fn states(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("state")
    }
    #[must_use]
    pub fn replays(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("replay")
    }
    #[must_use]
    pub fn creators(&self) -> meilisearch_sdk::indexes::Index {
        self.meili.index("creator")
    }
}
