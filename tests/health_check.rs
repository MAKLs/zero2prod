use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let url = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{url}/health"))
        .send()
        .await
        .expect("should have executed request");

    assert!(
        response.status().is_success(),
        "response should have been good"
    );
    assert_eq!(
        Some(0),
        response.content_length(),
        "response should have been empty"
    );
}

fn spawn_app() -> String {
    let addr = "127.0.0.1";
    let listener = TcpListener::bind((addr, 0)).expect("listener should have bound");
    let port = listener.local_addr().unwrap().port();
    let server =
        zero2prod::run_with_listener(listener).expect("server should have started listening");

    tokio::spawn(server);

    format!("http://{addr}:{port}")
}
