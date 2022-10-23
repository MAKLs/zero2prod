use std::net::TcpListener;

use reqwest::{header::CONTENT_TYPE, StatusCode};
use sqlx::{Connection, PgConnection};
use zero2prod::configuration::get_configuration;

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

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Setup
    let url = spawn_app();
    let configuration = get_configuration().expect("should have gotten configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("should have connected to database");
    let client = reqwest::Client::new();

    // Make request
    let body = "name=le%20gui&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{url}/subscriptions"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("should have executed request");

    // Assert response
    assert_eq!(
        StatusCode::OK,
        response.status(),
        "status code should be OK"
    );

    // Assert mutations
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("should have fetched saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let url = spawn_app();
    let client = reqwest::Client::new();

    let cases = vec![
        ("name=le%20gui", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (body, reason) in cases {
        let response = client
            .post(format!("{url}/subscriptions"))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("should have executed request");

        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "status code should be 400 because {reason}"
        );
    }
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
