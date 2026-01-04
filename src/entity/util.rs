use serde::Deserialize;

// The raw query params coming from the URL
// ?page=1&limit=15&disable_limit=true
#[derive(Debug, Deserialize, Default, Clone)]
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
