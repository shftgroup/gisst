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
use gisst::models::{CoreFileRole, ObjectRole};
use serde::Deserialize;
use serde_json::json;
use server_types::embed::PlayerStartTemplateInfo;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const CHECK_INTERVAL: u64 = 10;

#[tokio::main]
async fn main() -> Result<()> {
    use reqwest::ClientBuilder;
    let args = WorkerArgs::parse();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", args.worker_key))?,
    );
    let client = ClientBuilder::new()
        .tls_danger_accept_invalid_certs(args.allow_invalid_certs)
        .default_headers(headers)
        .build()
        .unwrap();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
    let cache = Path::new(&args.corefile_cache);
    loop {
        if let Err(e) = process_one_task(&client, cache, &args.gisst_url).await {
            log::error!("Error claiming task: {e:?}");
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
) -> Result<()> {
    use std::io::Write;
    let dest = local_root.join(local_path);
    if let Ok(true) = std::fs::exists(&dest) {
        println!("use cached {}", dest.display());
        return Ok(());
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
    Ok(())
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub task_id: Uuid,
    pub task_created_on: DateTime<Utc>,
    pub task_retry_count: i32,
    pub task_type: String,
    pub task_input: serde_json::Value,
}

async fn process_one_task(client: &reqwest::Client, cache: &Path, gisst_url: &str) -> Result<()> {
    let claim_url = format!("{gisst_url}/tasks/claim");
    let resp = match client
        .post(claim_url)
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
    if let Err(e) = run_task(client, cache, gisst_url, &task_data).await {
        log::error!("Error processing task: {e:?}, {:?}", e.backtrace());
        post_error(
            client,
            gisst_url,
            &task_data,
            &json!({"error":format!("{e:#}")}),
        )
        .await
    } else {
        Ok(())
    }
}
async fn run_task(
    client: &reqwest::Client,
    cache: &Path,
    gisst_url: &str,
    task_data: &Task,
) -> Result<()> {
    let task: server_types::embed::EmbedDataInfo =
        serde_json::from_value(task_data.task_input.clone())?;
    let cache_dir = cache
        .join(&task.environment.environment_platform)
        .join(&task.environment.environment_core_name)
        .join(&task.environment.environment_core_version);
    std::fs::create_dir_all(&cache_dir)?;
    let content_cache_dir = cache
        .join(&task.environment.environment_platform)
        .join(task.instance.instance_id.to_string());
    std::fs::create_dir_all(&content_cache_dir)?;
    let (corefile_name, main_content_name, replay_dest_path) = download_files(
        client,
        gisst_url,
        task_data,
        &task,
        &cache_dir,
        &content_cache_dir,
    )
    .await?;
    // 5. call video generator program
    let output_file = spawn_video_task(
        client,
        gisst_url,
        task_data,
        &cache_dir,
        &corefile_name,
        &content_cache_dir,
        &main_content_name,
        &replay_dest_path,
    )
    .await?;
    dbg!(output_file);
    // 6. upload the video via TUS to gisst-server (should the task include a file object UUID/WIP tus upload to upload to?)
    dbg!(&task);
    Err(anyhow!("not yet implemented"))
}

async fn download_files(
    client: &reqwest::Client,
    gisst_url: &str,
    task_data: &Task,
    task: &server_types::embed::EmbedDataInfo,
    cache_dir: &Path,
    content_cache_dir: &Path,
) -> Result<(String, String, PathBuf)> {
    use futures::{StreamExt, future::Either};
    let PlayerStartTemplateInfo::Replay(replay) = &task.start else {
        return Err(anyhow!("Task start is not a replay {:?}", task.start));
    };
    let replay_dir = std::env::temp_dir().join("ra-video-worker-replays");
    std::fs::create_dir_all(&replay_dir)?;
    let replay_path = format!("{}.replay", replay.replay_id);
    let replay_dest_path = replay_dir.join(&replay_path);
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
        cache_dir.to_path_buf(),
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
            cache_dir.join(&coredep.file_source_path),
            &coredep.file_filename,
        ));
    }
    // 3. download content
    let main_content = task
        .manifest
        .iter()
        .find(|l| l.object_role == ObjectRole::Content)
        .ok_or(anyhow!("no main content file"))?
        .file_filename
        .clone();
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

    let results = futures::stream::once(futures::future::try_join_all(downloads)).map(Either::Left);
    let status_update = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
        std::time::Duration::from_secs(180),
    ))
    .map(|_| Either::Right(()));
    let mut updates = futures::stream::select(results, status_update);
    loop {
        // TODO: use the futuresunordered stream of downloads to report download progress by file count
        match updates.select_next_some().await {
            Either::Left(Err(e)) => {
                return Err(anyhow!("Download error:{e:?}"));
            }
            Either::Left(Ok(_)) => {
                return Ok((
                    corefile.file_filename.clone(),
                    main_content,
                    replay_dest_path,
                ));
            }
            Either::Right(()) => {
                // post status update
                post_status(
                    client,
                    gisst_url,
                    task_data,
                    &json!({"phase":"downloading"}),
                )
                .await?;
            }
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "status")]
enum StatusUpdate {
    Error {
        error: String,
        cause: serde_json::Value,
    },
    Progress {
        frame: usize,
        total: Option<usize>,
        phase: String,
    },
    Finished {},
}

#[expect(clippy::too_many_arguments)]
async fn spawn_video_task(
    client: &reqwest::Client,
    gisst_url: &str,
    task_data: &Task,
    system_cache_dir: &Path,
    core_file: &str,
    content_cache_dir: &Path,
    main_content_file: &str,
    replay_path: &Path,
) -> Result<PathBuf> {
    use std::time::Instant;
    use tokio::io::{AsyncBufReadExt, BufReader};
    let out_path = replay_path.with_extension("webm");
    let mut bin_path = std::env::current_exe()?;
    bin_path.set_file_name("ra-video-genvideo");
    let mut child = tokio::process::Command::new(bin_path)
        .args([
            replay_path
                .to_str()
                .ok_or(anyhow!("Could not get replay path as unicode"))?,
            out_path
                .to_str()
                .ok_or(anyhow!("Could not get output path as unicode"))?,
            // TODO: more sophisticated handling of system files, bios, etc? retro-rs defaults to finding them next to core
            system_cache_dir
                .join(core_file)
                .to_str()
                .ok_or(anyhow!("Could not get core path as unicode"))?,
            content_cache_dir
                .join(main_content_file)
                .to_str()
                .ok_or(anyhow!("Could not get main content path as unicode"))?,
        ])
        .current_dir(content_cache_dir)
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| anyhow!("Could not create task: {e:?}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or(anyhow!("child did not have a handle to stdout"))?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    // TODO: replace with a reader that I can actually append to and a serde_json streamdeserializer
    let mut json_text = String::new();
    let mut last_posted = Instant::now();
    loop {
        tokio::select! {
            // not cancellation safe, but that's ok because the only other branch is the end of the process
            result = stdout_reader.next_line() => {
                match result {
                    Ok(None) => {},
                    Ok(Some(line)) => {
                        json_text.push_str(&line);
                    },
                    Err(_) => return Err(anyhow!("unexpected eof on stdout")),
                }
                match serde_json::from_str::<StatusUpdate>(&json_text) {
                    Ok(StatusUpdate::Error{error,cause}) => {
                        return Err(anyhow!("Video generation error {error} : {cause:?}"));
                    },
                    Ok(StatusUpdate::Progress{frame,total,phase}) => {
                        if last_posted.elapsed().as_secs() >= 180 {
                            post_status(client, gisst_url, task_data, &json!({"phase":phase, "frame":frame, "total":total})).await?;
                            last_posted = Instant::now();
                        }
                        json_text.clear();
                    }
                    Ok(StatusUpdate::Finished{}) => {
                        json_text.clear();
                        // just wait for process to exit
                    }
                    Err(e) if e.is_eof() => {
                        /* do not clear json_text, we need more lines of output before parsing status or error update */
                    },
                    Err(e) => {
                        return Err(anyhow!("Video generation error {e:?} on {json_text}"));
                    }
                }
            }
            result = child.wait() => {
                match result {
                    Ok(exit_code) if exit_code.success() => return Ok(out_path),
                    Ok(exit_code) => return Err(anyhow!("Process exited with code {exit_code}")),
                    Err(e) => return Err(anyhow!(e))
                }
            }
        };
    }
}
async fn post_status(
    client: &reqwest::Client,
    gisst_url: &str,
    task_data: &Task,
    json: &serde_json::Value,
) -> Result<()> {
    post_update(client, gisst_url, task_data, json, "status").await
}
async fn post_error(
    client: &reqwest::Client,
    gisst_url: &str,
    task_data: &Task,
    json: &serde_json::Value,
) -> Result<()> {
    post_update(client, gisst_url, task_data, json, "error").await
}
#[expect(unused)]
async fn post_complete(
    client: &reqwest::Client,
    gisst_url: &str,
    task_data: &Task,
    json: &serde_json::Value,
) -> Result<()> {
    post_update(client, gisst_url, task_data, json, "complete").await
}
async fn post_update(
    client: &reqwest::Client,
    gisst_url: &str,
    task_data: &Task,
    json: &serde_json::Value,
    status_code: &str,
) -> Result<()> {
    let status_url = format!("{gisst_url}/tasks/{}/{status_code}", task_data.task_id);
    println!("post {status_url} json {json:?}");
    client
        .post(status_url)
        .header("Content-Encoding", "application/json")
        .json(json)
        .send()
        .await?;
    Ok(())
}
