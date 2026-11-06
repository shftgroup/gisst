// poll for tasks with worker id
// claim task
//   get native core corresponding to emulator environment (best effort, or give a mapping)
//   get content file (store in cache)
//   get replay file
//   execute replay in core and dump as we go
//   upload video file
//   update replay record with video file
//   finish task
mod args;
use anyhow::{Result, anyhow};
use args::WorkerArgs;
use chrono::{DateTime, Utc};
use clap::Parser;
use gisst::models::CoreFileRole;
use serde::Deserialize;
use serde_json::json;
use server_types::embed::PlayerStartTemplateInfo;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const CHECK_INTERVAL: u64 = 10;

#[tokio::main]
async fn main() {
    use reqwest::ClientBuilder;
    let args = WorkerArgs::parse();
    let client = ClientBuilder::new()
        .tls_danger_accept_invalid_certs(args.allow_invalid_certs)
        .build()
        .unwrap();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
    let claim_url = format!("{}/tasks/claim", args.gisst_url);
    let auth_header = format!("Bearer {}", args.worker_key);
    let cache = Path::new(&args.corefile_cache);
    loop {
        if let Err(e) =
            process_one_task(&client, cache, &args.gisst_url, &claim_url, &auth_header).await
        {
            log::error!("Error processing task: {e:?}");
        }
        tokio::time::sleep(std::time::Duration::from_secs(CHECK_INTERVAL)).await;
    }
}

async fn download_file(
    client: &reqwest::Client,
    gisst_url: &str,
    path: &str,
    local_root: PathBuf,
    local_path: &str,
) -> Result<PathBuf> {
    use std::io::Write;
    let dest = local_root.join(local_path);
    if let Ok(true) = std::fs::exists(&dest) {
        println!("use cached {}", dest.display());
        return Ok(dest);
    }
    println!("download {gisst_url}/storage/{path} to {}", dest.display());
    let mut resp = client
        .get(format!("{gisst_url}/storage/{path}"))
        .send()
        .await?;
    let mut file = std::fs::File::create_new(&dest)?;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk)?;
    }
    Ok(dest)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub task_id: Uuid,
    pub task_created_on: DateTime<Utc>,
    pub task_retry_count: i32,
    pub task_type: String,
    pub task_input: serde_json::Value,
}

async fn process_one_task(
    client: &reqwest::Client,
    cache: &Path,
    gisst_url: &str,
    claim_url: &str,
    auth_header: &str,
) -> Result<()> {
    let resp = match client
        .post(claim_url)
        .header("Authorization", auth_header)
        .header("Content-Encoding", "application/json")
        .json(&json!({"task_type":"retroarch_replay_video"}))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => resp,
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => return Ok(()),
        Ok(resp) => {
            return Err(anyhow!(
                "task fetch error:{:?}",
                resp.error_for_status().err()
            ));
        }
        Err(e) if matches!(e.status(), Some(reqwest::StatusCode::NOT_FOUND)) => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    let task_data: Task = dbg!(resp.json().await?);
    let task: server_types::embed::EmbedDataInfo = serde_json::from_value(task_data.task_input)?;
    let cache_dir = cache
        .join(&task.environment.environment_platform)
        .join(&task.environment.environment_core_name)
        .join(&task.environment.environment_core_version);
    std::fs::create_dir_all(&cache_dir)?;
    let content_cache_dir = cache
        .join(&task.environment.environment_platform)
        .join(task.instance.instance_id.to_string());
    std::fs::create_dir_all(&content_cache_dir)?;
    let PlayerStartTemplateInfo::Replay(replay) = &task.start else {
        return Err(anyhow!("Task start is not a replay {:?}", task.start));
    };
    let replay_dir = std::env::temp_dir().join("ra-video-worker-replays");
    std::fs::create_dir_all(&replay_dir)?;
    let replay_path = format!("{}.replay", replay.replay_id);
    // 1. use the task's core manifest to find the appropriate native core for this OS/arch
    let suffix = format!(
        "_libretro.{}.{}",
        std::env::consts::ARCH,
        std::env::consts::DLL_EXTENSION
    );
    let corefile = task
        .core_manifest
        .iter()
        .find(|l| l.core_role == CoreFileRole::Native && l.file_filename.ends_with(&suffix))
        .ok_or(anyhow!("No native core for task {task:?}"))?;
    // 2. download that native core, and the dependencies and configs (not entrypoint) for the core
    let prom = download_file(
        client,
        gisst_url,
        &corefile.file_dest_path,
        cache_dir.clone(),
        &corefile.file_filename,
    );
    let mut downloads = vec![prom];
    for coredep in task
        .core_manifest
        .iter()
        .filter(|l| l.core_role == CoreFileRole::Dependency || l.core_role == CoreFileRole::Config)
    {
        downloads.push(download_file(
            client,
            gisst_url,
            &coredep.file_dest_path,
            cache.join(&coredep.file_source_path),
            &coredep.file_filename,
        ));
    }
    // 3. download content
    for file in &task.manifest {
        downloads.push(download_file(
            client,
            gisst_url,
            &file.file_dest_path,
            content_cache_dir.join(&file.file_source_path),
            &file.file_filename,
        ));
    }
    // 4. download start data (must be replay)
    downloads.push(download_file(
        client,
        gisst_url,
        &replay.file_dest_path,
        replay_dir,
        &replay_path,
    ));
    // Complete all the downloads we need before generating video
    let results = futures::future::join_all(downloads).await;
    let errs: Vec<_> = results.into_iter().filter_map(Result::err).collect();
    if !errs.is_empty() {
        return Err(anyhow!("Download errors:{errs:?}"));
    }
    // 5. proceed as in the rply-codec genvideo example -- but use glgfx and copy_framebuffer_rgba8888.
    //   maybe: need to fix retro-rs to fall back to software framebuffer if GL not requested
    // 6. upload the video via TUS to gisst-server (should the task include a file object UUID/WIP tus upload to upload to?)
    dbg!(&task);
    Err(anyhow!("not yet implemented"))
}
