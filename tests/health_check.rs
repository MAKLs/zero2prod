use std::net::TcpListener;

use once_cell::sync::Lazy;
use reqwest::{header::CONTENT_TYPE, StatusCode};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing::Subscriber;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    // Set up test logging
    let default_filter = "info";
    let subscriber_name = "test";
    // TODO box the sink and create the subscriber only once
    let subscriber = if std::env::var("TEST_LOG").is_ok() {
        Box::new(get_subscriber(
            subscriber_name,
            default_filter,
            std::io::stdout,
        )) as Box<dyn Subscriber + Send + Sync>
    } else {
        Box::new(get_subscriber(
            subscriber_name,
            default_filter,
            std::io::sink,
        )) as Box<dyn Subscriber + Send + Sync>
    };
    init_subscriber(subscriber);
});

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

async fn spawn_app() -> TestApp {
    // Force logging initialization for testing
    Lazy::force(&TRACING);
    let address = "127.0.0.1";
    let listener = TcpListener::bind((address, 0)).expect("listener should have bound");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://{address}:{port}");
    let mut configuration = get_configuration().expect("should have gotten configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;
    let server = zero2prod::run_with_listener(listener, connection_pool.clone())
        .expect("server should have started listening");

    tokio::spawn(server);

    TestApp {
        address,
        pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("should have connected to postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("should have created database");

    // Run migration
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("should have created database pool");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("should have run migrations");

    connection_pool
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
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
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
        ("name=le%20guin", "missing the email"),
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
