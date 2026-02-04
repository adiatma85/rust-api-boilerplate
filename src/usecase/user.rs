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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use sea_orm::prelude::DateTimeUtc;

    use super::*;
    use crate::domain::user::MockUserDomainTrait;

    fn create_test_user() -> user::Model {
        user::Model {
            id: 1,
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            hashed_password: "hashed_password".to_string(),
            status: 1,
            created_at: DateTimeUtc::from(Utc::now()),
            updated_at: DateTimeUtc::from(Utc::now()),
        }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let mut mock_user_domain = MockUserDomainTrait::new();
        let user = create_test_user();

        mock_user_domain
            .expect_create()
            .times(1)
            .returning(move |_| Ok(user.clone()));

        let user_usecase = init(InitParam {
            user_domain: Arc::new(mock_user_domain),
        });

        let result = user_usecase
            .create_user(CreateUserUseParam {
                name: "Test User".to_string(),
                email: "test@example.com".to_string(),
                password: "password123".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let created_user = result.unwrap();
        assert_eq!(created_user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_get_list_user_success() {
        let mut mock_user_domain = MockUserDomainTrait::new();
        let user = create_test_user();
        let pagination = Pagination {
            current_page: 1,
            current_elements: 1,
            total_pages: 1,
            total_elements: 1,
            sort_by: vec![],
        };

        mock_user_domain
            .expect_get_list()
            .times(1)
            .returning(move |_| Ok((vec![user.clone()], pagination.clone())));

        let user_usecase = init(InitParam {
            user_domain: Arc::new(mock_user_domain),
        });

        let result = user_usecase.get_list_user(UserDomParam::default()).await;

        assert!(result.is_ok());
        let (users, _) = result.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].email, Some("test@example.com".to_string()));
    }
}
