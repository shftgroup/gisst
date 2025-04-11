use reqwest;
use tokio;
use tokio::time::sleep;
use assert_cmd::cargo::CommandCargoExt;
use std::process::Stdio;
use std::{thread, time};
use std::io::Read;
use std::process::{Command,Child};

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // You can check std::thread::panicking() here
        match self.0.kill() {
            Err(e) => println!("Could not kill child process: {}", e),
            Ok(_) => println!("Successfully killed child process"),
        }
    }
}


#[tokio::test]
async fn test_async_main() {
    let child = Command::cargo_bin("gisst-server")
        .expect("Couldn't get gisst binary")
        .current_dir("..")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start sever");
    
    let guard = ChildGuard(child);

    tokio::time::sleep(time::Duration::from_millis(5000)).await;

    let client = reqwest::Client::builder()
        .user_agent("servertest")
        .build()
        .unwrap();

    
    
    let res = client.get("https://localhost:3000/").send().await.unwrap();
    
    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("Login"));
    assert!(body.contains("Request Access"));
}


// #[tokio::test]
// async fn test_authenticated_instances_page() {
//     let config = ServerConfig::new().unwrap();

//     Command::cargo_bin("gisst-server")
//         .args(["run", "--bin", "gisst-server", "--features", "dummy_auth"])
//         .spawn()
//         .expect("process failed to start");


//     let client = reqwest::Client::builder()
//         .cookie_store(true) 
//         .build()
//         .unwrap();

//     let login_url = "https://localhost:3000/login?code=verysecret";
//     let login_res = client.get(login_url).send().await.unwrap();

//     assert_eq!(login_res.status(), 302);

//     let res = client
//         .get("https://localhost:3000/instances")
//         .send()
//         .await
//         .unwrap();

//     assert_eq!(res.status(), 200);

//     let body = res.text().await.unwrap();
//     assert!(body.contains("Instances"));
//     assert!(body.contains("Log out"));
// }
