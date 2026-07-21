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
use args::WorkerArgs;
use clap::Parser;
use serde_json::json;
use anyhow::{Context, Result};

const CHECK_INTERVAL:u64 = 60;

#[tokio::main]
async fn main() {
    use reqwest::Client;
    let client = Client::new();
    let args = WorkerArgs::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
    let claim_url = format!("{}/tasks/claim", args.gisst_url);
    let auth_header = format!("Bearer {}", args.worker_key);
    loop {
        tokio::time::sleep(std::duration::Duration::from_secs(CHECK_INTERVAL)).await;
        if let Err(e) = process_one_task(&client, &claim_url, &auth_header).await {
            info::error!("Error processing task: {e:?}");
        }
    }
}

async fn process_one_task(client:&reqwest::Client, claim_url:&str, auth_header:&str) -> Result<()> {
    let resp = match client
        .post(&claim_url)
        .header("Authorization", &auth_header)
        .header("Content-Encoding", "application/json")
        .json(&json!({"task_type":"retroarch_replay_video"}))
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) if matches!(e.status(), Some(reqwest::StatusCode::NOT_FOUND)) => return Ok(()),
            Err(e) => return Err(e)
        };
    let task:serde_json::Value = resp.json().await;
    
    dbg!(task);
}
