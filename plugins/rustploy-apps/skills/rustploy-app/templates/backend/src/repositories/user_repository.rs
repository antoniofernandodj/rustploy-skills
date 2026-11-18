use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppResult, models::User};

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

const COLUMNS: &str = "id, email, name, password_hash, role, active, created_at";

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(&format!("SELECT {COLUMNS} FROM users WHERE email = $1"))
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(&format!("SELECT {COLUMNS} FROM users WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    /// Cria o usuário se o e-mail ainda não existe. Devolve `true` se criou.
    /// Nunca altera um usuário existente (não reseta senha trocada).
    pub async fn insert_if_absent(&self, email: &str, name: &str, password_hash: &str, role: &str) -> AppResult<bool> {
        let result = sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, role) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (email) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .bind(role)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }
}
