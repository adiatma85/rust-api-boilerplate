use crate::business::usecase::Usecase;

#[derive(Clone)]
pub struct AppState {
    // We will use owned usecase in here
    pub usecase: Usecase,

    // JWT Secret in here temporarily
    pub jwt_secret: String,

    pub service_version: String,
}

pub struct AppStateInitParam {
    pub secret_key: String,
    pub usecase: Usecase,
    pub service_version: String,
}

impl AppState {
    pub fn new(param: AppStateInitParam) -> Self {
        Self {
            usecase: param.usecase,
            jwt_secret: param.secret_key,
            service_version: param.service_version,
        }
    }
}
