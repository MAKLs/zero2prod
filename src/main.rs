#![forbid(unsafe_code)]
use zero2prod::{configuration::get_configuration, run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("failed to read configuration");
    run(("127.0.0.1", configuration.port))?.await
}
