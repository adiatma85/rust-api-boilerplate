use crate::entity::general::General;

pub struct User {
    pub id: u32,
    pub email: String,
    pub name: String,
    pub hashed_password: String,

    // General Fields
    pub general: General,
}
