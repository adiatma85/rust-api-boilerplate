use serde::Deserialize;

use crate::config::{
    creds::Credentials,
    database::{DatabaseConfig, DatabaseProvider},
};

#[derive(Debug, Deserialize)]
pub struct AppMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct AppSettings {
    pub app_metadata: AppMetadata,
    pub database: DatabaseConfig,
    pub creds: Credentials,
}

impl AppSettings {
    pub fn new(config_path: &str) -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            // 1. Read from a config file (e.g., config.json)
            // "required(false)" means it won't crash if the file is missing
            .add_source(config::File::with_name(config_path).required(true))
            // 2. Read from Environment Variables
            // This allows overriding via "APP_PORT=9090"
            // The prefix "APP" helps avoid collisions
            .add_source(config::Environment::with_prefix("APP").separator("__"));

        // Build and deserialize into our struct
        builder.build()?.try_deserialize()
    }

    // 3. Helper to build the connection string (DSN)
    pub fn database_url(&self) -> String {
        match self.database.provider {
            DatabaseProvider::Mysql => format!(
                "mysql://{}:{}@{}:{}/{}",
                self.database.username,
                self.database.password,
                self.database.host,
                self.database.port,
                self.database.name
            ),
            // We can implement others later
            _ => unimplemented!("Provider not supported yet"),
        }
    }

    // --- We can build additional logic below in here ---
}
