use serde::Deserialize;

use crate::config::database::DatabaseProvider;

#[derive(Deserialize)]
pub struct AppSettings {
    pub port: u16,

    // Database Configs
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub db_provider: DatabaseProvider,
}

impl AppSettings {
    pub fn new(config_path: &str) -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            // 1. Start with default values (optional)
            .set_default("port", 8080)?
            // 2. Read from a config file (e.g., config.json)
            // "required(false)" means it won't crash if the file is missing
            .add_source(config::File::with_name(config_path).required(true))
            // 3. Read from Environment Variables
            // This allows overriding via "APP_PORT=9090"
            // The prefix "APP" helps avoid collisions
            .add_source(config::Environment::with_prefix("APP").separator("__"));

        // Build and deserialize into our struct
        builder.build()?.try_deserialize()
    }

    // 3. Helper to build the connection string (DSN)
    pub fn database_url(&self) -> String {
        match self.db_provider {
            DatabaseProvider::Mysql => format!(
                "mysql://{}:{}@{}:{}/{}",
                self.db_user, self.db_password, self.db_host, self.db_port, self.db_name
            ),
            // We can implement others later
            _ => unimplemented!("Provider not supported yet"),
        }
    }
}
