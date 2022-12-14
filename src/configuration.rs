use secrecy::{ExposeSecret, Secret};

/// Application settings
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    /// Server port
    pub port: u16,
}

/// Database settings
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    /// Username to connect to database
    pub username: String,
    /// Password of user for username.
    pub password: Secret<String>,
    /// Port database is listening on
    pub port: u16,
    /// Host database is at
    pub host: String,
    /// Name of database
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    settings.try_deserialize()
}
