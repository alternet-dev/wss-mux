use crate::common::{spawn_server, test_state, test_state_with_manifest};

#[tokio::test]
async fn ready_is_503_before_manifest() {
    let addr = spawn_server(test_state()).await;
    let resp = reqwest::get(format!("http://{addr}/ready"))
        .await
        .expect("request");
    assert_eq!(resp.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    let body = resp.text().await.expect("body");
    assert_eq!(body, "not ready");
}

#[tokio::test]
async fn ready_is_200_after_manifest() {
    let addr = spawn_server(test_state_with_manifest()).await;
    let resp = reqwest::get(format!("http://{addr}/ready"))
        .await
        .expect("request");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = resp.text().await.expect("body");
    assert_eq!(body, "ready");
}
