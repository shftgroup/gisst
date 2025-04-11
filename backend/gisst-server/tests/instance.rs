use gisst_server::serverconfig::ServerConfig;
use reqwest;
use tokio;
use std::process::Command;

#[tokio::test]
async fn test_async_main() {
    let res = reqwest::get("https://localhost:3000").await.expect("Request failed");
    
    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("Login"));
    assert!(body.contains("Request Access"));
}


#[tokio::test]
async fn test_authenticated_instances_page() {
    
    Command::new("cargo")
        .env("GISST_ENV", ServerConfig)
        .args(["run", "--bin", "gisst-server", "--features", "dummy_auth"])
        .spawn()
        .expect("process failed to start");


    let client = reqwest::Client::builder()
        .cookie_store(true) 
        .build()
        .unwrap();

    let login_url = "https://localhost:3000/login?code=verysecret";
    let login_res = client.get(login_url).send().await.unwrap();

    assert_eq!(login_res.status(), 302);

    let res = client
        .get("https://localhost:3000/instances")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    let body = res.text().await.unwrap();
    assert!(body.contains("Instances"));
    assert!(body.contains("Log out"));
}
