use reqwest;
use tokio;

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
