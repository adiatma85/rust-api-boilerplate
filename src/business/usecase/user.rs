use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, error, info};

use crate::{
    business::{
        domain::user::UserDomainTrait,
        entity::{
            response::{AppCode, Pagination},
            user::{self, CreateUserDomParam, CreateUserUseParam, UserDomParam, UserUseResponse},
        },
    },
    helper as crate_helper,
};

#[async_trait]
pub trait UserUsecaseTrait: Send + Sync {
    async fn create_user(&self, params: CreateUserUseParam) -> Result<user::Model, AppCode>;
    async fn get_list(
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
        debug!("create user attempt for email: {}", params.email);

        let hashed_pwd = crate_helper::password::hash_password(&params.password).map_err(|e| {
            error!(
                "failed to hash password for user: {}, error: {:?}",
                params.email, e
            );
            AppCode::from(e)
        })?;

        let repo_params = CreateUserDomParam {
            name: params.name.clone(),
            email: params.email.clone(),
            hashed_password: hashed_pwd,
        };

        let created_user = self.user_domain.create(repo_params).await.map_err(|e| {
            error!("failed to create user: {}, error: {:?}", params.email, e);
            AppCode::from(e)
        })?;

        info!(
            "user created successfully: {} (id: {})",
            created_user.email, created_user.id
        );
        Ok(created_user)
    }

    async fn get_list(
        &self,
        params: UserDomParam,
    ) -> Result<(Vec<UserUseResponse>, Pagination), AppCode> {
        debug!("get user list request: {:?}", params);

        let result = self.user_domain.get_list(params).await;

        // This will change the Error AppError to AppCode instead
        match result {
            Ok((users, pagination)) => {
                debug!("original users retrieved: {:?}", &users);
                let user_responses = users
                    .into_iter()
                    .map(|user| UserUseResponse {
                        id: Some(user.id),
                        email: Some(user.email),
                    })
                    .collect();

                debug!("user responses prepared: {:?}", &user_responses);

                info!(
                    "user list retrieved successfully: {} users (page: {}/{})",
                    pagination.current_elements, pagination.current_page, pagination.total_pages
                );

                Ok((user_responses, pagination))
            }
            Err(err) => {
                error!("failed to retrieve user list: {:?}", err);
                Err(AppCode::from(err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use sea_orm::prelude::DateTimeUtc;

    use super::*;
    use crate::business::domain::user::MockUserDomainTrait;

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

        let result = user_usecase.get_list(UserDomParam::default()).await;

        assert!(result.is_ok());
        let (users, _) = result.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].email, Some("test@example.com".to_string()));
    }
}
