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
    pub password: String,
    /// Port database is listening on
    pub port: u16,
    /// Host database is at
    pub host: String,
    /// Name of database
    pub database_name: String,
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
