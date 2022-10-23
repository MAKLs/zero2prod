use std::net::TcpListener;

use reqwest::{header::CONTENT_TYPE, StatusCode};
use sqlx::PgPool;
use zero2prod::configuration::get_configuration;

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let address = "127.0.0.1";
    let listener = TcpListener::bind((address, 0)).expect("listener should have bound");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://{address}:{port}");
    let configuration = get_configuration().expect("should have gotten configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("should have create connection pool");
    let server = zero2prod::run_with_listener(listener, connection_pool.clone())
        .expect("server should have started listening");

    tokio::spawn(server);

    TestApp {
        address,
        pool: connection_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health", app.address))
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
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Make request
    let body = "name=le%20gui&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", app.address))
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
        .fetch_one(&app.pool)
        .await
        .expect("should have fetched saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let cases = vec![
        ("name=le%20gui", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (body, reason) in cases {
        let response = client
            .post(format!("{}/subscriptions", app.address))
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
