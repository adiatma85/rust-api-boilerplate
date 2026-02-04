use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

// The raw query params coming from the URL
// ?page=1&limit=15&disable_limit=true
#[derive(Debug, Deserialize, Default, Clone, IntoParams, ToSchema, PartialEq)]
#[serde(default)] // Allows fields to be missing (uses defaults)
pub struct PaginationParams {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub disable_limit: bool,
}

// The Trait that fetch_list_2 will rely on
pub trait Paginatable {
    fn get_page(&self) -> u64;
    fn get_limit(&self) -> u64;
    fn is_limit_disabled(&self) -> bool;
}
