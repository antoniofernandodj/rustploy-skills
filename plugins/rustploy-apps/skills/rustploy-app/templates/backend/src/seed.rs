//! Usuário inicial. Idempotente: cria o admin só se o e-mail ainda não existe e
//! nunca altera um usuário existente. E-mail e senha vêm do ambiente (secret
//! ADMIN_PASSWORD no rustploy); nada fica no código.
use crate::{config, repositories::UserRepository, security};

pub struct AdminSeed {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl AdminSeed {
    pub fn from_env() -> anyhow::Result<Self> {
        let password = config::required("ADMIN_PASSWORD")?;
        if password.chars().count() < 12 {
            anyhow::bail!("ADMIN_PASSWORD precisa ter pelo menos 12 caracteres");
        }
        Ok(Self {
            email: config::required("ADMIN_EMAIL")?.trim().to_lowercase(),
            name: config::optional("ADMIN_NAME", "Administrador"),
            password,
        })
    }
}

pub async fn run(pool: &sqlx::PgPool, admin: &AdminSeed) -> anyhow::Result<()> {
    let users = UserRepository::new(pool.clone());
    let hash = security::hash_password(&admin.password)?;
    let created = users.insert_if_absent(&admin.email, &admin.name, &hash, "admin").await?;
    if created {
        tracing::info!(email = %admin.email, "usuário inicial criado");
    } else {
        tracing::info!(email = %admin.email, "usuário inicial já existia; nada alterado");
    }
    Ok(())
}
