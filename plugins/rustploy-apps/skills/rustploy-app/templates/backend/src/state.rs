//! Estado compartilhado: monta repositories e services uma vez, na subida.
use std::sync::Arc;

use crate::{config::Config, repositories::UserRepository, services::AuthService};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Arc<Config>,
    pub auth_service: Arc<AuthService>,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool, config: Config) -> Self {
        let config = Arc::new(config);
        let user_repository = UserRepository::new(pool.clone());
        let auth_service = Arc::new(AuthService::new(user_repository, config.clone()));
        Self { pool, config, auth_service }
    }
}
