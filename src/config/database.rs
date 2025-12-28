use serde::Deserialize;

// 1. Define the Database Provider Enum
// We use `#[serde(rename_all = "lowercase")]` so "mysql" in JSON maps to this Enum
#[derive(Debug, Deserialize, Clone)] // Clone is useful for passing settings around
#[serde(rename_all = "lowercase")]
pub enum DatabaseProvider {
    Mysql,
    Postgres,
    Sqlite,
}
