use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    domain::user::UserDomainTrait,
    entity::{
        response::{AppCode, Pagination},
        user::{self, CreateUserDomParam, CreateUserUseParam, UserDomParam, UserUseResponse},
    },
    helper,
};

#[async_trait]
pub trait UserUsecaseTrait: Send + Sync {
    async fn create_user(&self, params: CreateUserUseParam) -> Result<user::Model, AppCode>;
    async fn get_list_user(
        &self,
        params: UserDomParam,
    ) -> Result<(Vec<UserUseResponse>, Pagination), AppCode>;
}

pub struct UserUsecase {
    user_domain: Arc<dyn UserDomainTrait>,
}

pub struct InitParam {
    pub user_domain: Arc<dyn UserDomainTrait>,
}

pub fn init(init_param: InitParam) -> impl UserUsecaseTrait {
    UserUsecase {
        user_domain: init_param.user_domain,
    }
}

// --- Implementation blocks ---

#[async_trait]
impl UserUsecaseTrait for UserUsecase {
    async fn create_user(&self, params: CreateUserUseParam) -> Result<user::Model, AppCode> {
        let hashed_pwd = helper::hash_password(&params.password).map_err(AppCode::from)?;

        let repo_params = CreateUserDomParam {
            name: params.name,
            email: params.email,
            hashed_password: hashed_pwd,
        };

        self.user_domain
            .create(repo_params)
            .await
            .map_err(AppCode::from)
    }

    async fn get_list_user(
        &self,
        params: UserDomParam,
    ) -> Result<(Vec<UserUseResponse>, Pagination), AppCode> {
        let result = self.user_domain.get_list(params).await;

        // This will change the Error AppError to AppCode instead
        match result {
            Ok((users, pagination)) => {
                println!("The original users are: {:?}", &users);
                let user_responses = users
                    .into_iter()
                    .map(|user| UserUseResponse {
                        id: Some(user.id),
                        email: Some(user.email),
                    })
                    .collect();

                println!("The users are: {:?}", &user_responses);

                Ok((user_responses, pagination))
            }
            Err(err) => Err(AppCode::from(err)),
        }
    }
}
