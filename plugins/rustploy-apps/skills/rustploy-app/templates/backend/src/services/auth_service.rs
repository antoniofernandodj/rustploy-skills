use std::sync::Arc;

use uuid::Uuid;

use crate::{
    config::Config,
    error::{AppError, AppResult},
    models::{LoginRequest, LoginResponse, User},
    repositories::UserRepository,
    security,
};

pub struct AuthService {
    users: UserRepository,
    config: Arc<Config>,
}

impl AuthService {
    pub fn new(users: UserRepository, config: Arc<Config>) -> Self {
        Self { users, config }
    }

    pub async fn login(&self, request: LoginRequest) -> AppResult<LoginResponse> {
        let email = request.email.trim().to_lowercase();
        if email.is_empty() || request.password.is_empty() {
            return Err(AppError::Validation("informe e-mail e senha".into()));
        }
        // Mesma resposta para "não existe" e "senha errada": não revela quais e-mails existem.
        let user = self.users.find_by_email(&email).await?.ok_or(AppError::Unauthorized)?;
        if !user.active || !security::verify_password(&request.password, &user.password_hash) {
            return Err(AppError::Unauthorized);
        }
        let token = security::issue_token(&user, &self.config)?;
        Ok(LoginResponse { token, user: user.into() })
    }

    pub async fn current_user(&self, user_id: Uuid) -> AppResult<User> {
        let user = self.users.find_by_id(user_id).await?.ok_or(AppError::Unauthorized)?;
        if !user.active {
            return Err(AppError::Unauthorized);
        }
        Ok(user)
    }
}
