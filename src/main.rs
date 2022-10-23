#![forbid(unsafe_code)]
use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("failed to read configuration");
    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("should have connected to database");
    run(("127.0.0.1", configuration.port), pool)?.await
}
