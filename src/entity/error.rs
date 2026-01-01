use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
}

// Map SeaORM -> AppError (So you can use '?' in services)
impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        match err {
            DbErr::RecordNotFound(_) => AppError::NotFound,
            _ => AppError::InternalServerError(err.to_string()),
        }
    }
}
