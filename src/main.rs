#![forbid(unsafe_code)]
use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::{
    configuration::get_configuration,
    run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber(env!("CARGO_BIN_NAME"), "info", std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("failed to read configuration");
    let pool = PgPool::connect(configuration.database.connection_string().expose_secret())
        .await
        .expect("should have connected to database");
    run(("127.0.0.1", configuration.port), pool)?.await
}
