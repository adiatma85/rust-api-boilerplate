use std::sync::Arc;

use crate::{
    domain::user::UserDomainTrait,
    entity::user::{CreateUserDomParam, CreateUserUseParam},
    usecase::auth::AuthUsecase,
};

pub struct UserUsecase {
    user_domain: Arc<dyn UserDomainTrait>,
}

pub struct UserUseInitParam {
    pub user_domain: Arc<dyn UserDomainTrait>,
}

impl UserUsecase {
    pub fn new(init_param: UserUseInitParam) -> Self {
        Self {
            user_domain: init_param.user_domain,
        }
    }

    pub async fn create_user(&self, params: CreateUserUseParam) -> Result<i32, String> {
        let hashed_pwd = AuthUsecase::hash_password(&params.password)?;

        let repo_params = CreateUserDomParam {
            name: params.name,
            email: params.email,
            hashed_password: hashed_pwd,
        };

        self.user_domain.create(repo_params).await
    }
}
