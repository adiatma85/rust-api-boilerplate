use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppSettings {
    pub port: u16,
}

impl AppSettings {
    pub fn new(config_path: &str) -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            // 1. Start with default values (optional)
            .set_default("port", 8080)?
            // 2. Read from a config file (e.g., config.json)
            // "required(false)" means it won't crash if the file is missing
            .add_source(config::File::with_name(config_path).required(false))
            // 3. Read from Environment Variables
            // This allows overriding via "APP_PORT=9090"
            // The prefix "APP" helps avoid collisions
            .add_source(config::Environment::with_prefix("APP").separator("__"));

        // Build and deserialize into our struct
        builder.build()?.try_deserialize()
    }
}
