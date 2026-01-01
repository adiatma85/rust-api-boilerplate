use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    domain::user::UserDomainTrait,
    entity::{
        response::AppCode,
        user::{CreateUserDomParam, CreateUserUseParam, UserDomParam, UserUseResponse},
    },
    usecase::auth::AuthUsecase,
};

#[async_trait]
pub trait UserUsecaseTrait: Send + Sync {
    async fn create_user(&self, params: CreateUserUseParam) -> Result<i32, String>;
    async fn get_list_user(
        &self,
        params: UserDomParam,
    ) -> Result<(Vec<UserUseResponse>, i64), AppCode>;
}

pub struct UserUsecase {
    user_domain: Arc<dyn UserDomainTrait>,
}

pub struct InitParam {
    pub user_domain: Arc<dyn UserDomainTrait>,
}

// --- Implementation blocks ---

impl UserUsecase {
    pub fn new(init_param: InitParam) -> Self {
        Self {
            user_domain: init_param.user_domain,
        }
    }
}

#[async_trait]
impl UserUsecaseTrait for UserUsecase {
    async fn create_user(&self, params: CreateUserUseParam) -> Result<i32, String> {
        let hashed_pwd = AuthUsecase::hash_password(&params.password)?;

        let repo_params = CreateUserDomParam {
            name: params.name,
            email: params.email,
            hashed_password: hashed_pwd,
        };

        self.user_domain.create(repo_params).await
    }

    async fn get_list_user(
        &self,
        params: UserDomParam,
    ) -> Result<(Vec<UserUseResponse>, i64), AppCode> {
        let result = self.user_domain.get_list(params).await;

        // This will change the Error AppError to AppCode instead
        match result {
            Ok((users, total)) => {
                println!("The original users are: {:?}", &users);
                let user_responses = users
                    .into_iter()
                    .map(|user| UserUseResponse {
                        id: Some(user.id),
                        email: Some(user.email),
                    })
                    .collect();

                println!("The users are: {:?}", &user_responses);

                Ok((user_responses, total))
            }
            Err(err) => Err(AppCode::from(err)),
        }
    }
}
