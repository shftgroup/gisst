use sqlx::PgPool;
use meilisearch_sdk::client::*;

use gisst::{
    models::{
        Creator, InstanceWork, Replay, Save, State, Work,
    },
};

use crate::GISSTCliError;

pub async fn populate(db: PgPool, url:&str, api_key:&str) -> Result<(), GISSTCliError> {
    use futures::{StreamExt,FutureExt,TryFutureExt,TryStreamExt};
    const CHUNK_SIZE:usize=10000;
    let client = Client::new(url, Some(api_key)).unwrap();
    {
        let idx = client.index("instance");
        sqlx::query_as!(
            InstanceWork,
            r#"SELECT work_id as "work_id!", work_name as "work_name!",
               work_version as "work_version!", work_platform as "work_platform!",
               instance_id as "instance_id!", row_num as "row_num!"
               FROM instancework"#
        ).fetch(&db).try_chunks(CHUNK_SIZE).for_each(|instances| async {idx.add_documents(&(instances.unwrap()), Some("instance_id")).await.unwrap();}).await;
    }
    // todo state, save, replay should include instancework data too
    // and i don't love all the sql in cli.
    // maybe: amend creatorreplayinfo/stateinfo/saveinfo to include freator metadata and give a way
    // in models.rs to get a stream instead of a vec
    // maybe theres a SearchProvider trait that the server can instantiate for meili, and pass that
    // into gisst crate for collecting documents to index and sending them to meili. then the sql
    // can stay in gisst, and it can also be used to send new documents as they are inserted (to
    // creator, instance, replay, state, save).
    // but then, how does cli get access to that indexer?  maybe MeiliSearchProvider is in gisst
    // but needs a url and a key to be made to submit.
    // and that must be a parameter to all inserts or updates on relevant types
    {
        let idx = client.index("state");
        sqlx::query_as!(
            State,
            r#"SELECT * FROM state"#
        ).fetch(&db).try_chunks(CHUNK_SIZE).for_each(|states| async {
            idx.add_documents(&(states.unwrap()), Some("state_id")).await.unwrap();
        }).await;
    }
    {
        let idx = client.index("replay");
        sqlx::query_as!(
            Replay,
            r#"SELECT * FROM replay"#
        ).fetch(&db).try_chunks(CHUNK_SIZE).for_each(|replays| async {
            idx.add_documents(&(replays.unwrap()), Some("replay_id")).await.unwrap();
        }).await;
    }
    {
        let idx = client.index("save");
        sqlx::query_as!(
            Save,
            r#"SELECT * FROM save"#
        ).fetch(&db).try_chunks(CHUNK_SIZE).for_each(|saves| async { idx.add_documents(&(saves.unwrap()), Some("save_id")).await.unwrap();}).await;
    }
    {
        // TODO maybe better to index something like "saves U states U replays with facet by creator"
        let idx = client.index("creator");
        sqlx::query_as!(
            Creator,
            r#"SELECT * FROM creator"#
        ).fetch(&db).try_chunks(CHUNK_SIZE).for_each(|creators| async {
idx.add_documents(&(creators.unwrap()), Some("creator_id")).await.unwrap();}).await;
    }

    Ok(())
}
