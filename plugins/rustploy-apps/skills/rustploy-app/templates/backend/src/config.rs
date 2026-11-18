//! Configuração lida do ambiente. Segredo ausente = o app NÃO sobe (sem valor padrão).
use anyhow::{bail, Context};

#[derive(Clone)]
pub struct Config {
    pub bind_address: String,
    pub jwt_secret: String,
    pub jwt_ttl_hours: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let jwt_secret = required("JWT_SECRET")?;
        if jwt_secret.len() < 32 {
            bail!("JWT_SECRET precisa ter pelo menos 32 caracteres");
        }
        Ok(Self {
            bind_address: optional("BIND_ADDRESS", "0.0.0.0:8080"),
            jwt_secret,
            jwt_ttl_hours: optional("JWT_TTL_HOURS", "8").parse().context("JWT_TTL_HOURS deve ser um número")?,
        })
    }
}

pub fn required(name: &str) -> anyhow::Result<String> {
    match std::env::var(name) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => bail!("variável de ambiente obrigatória ausente ou vazia: {name}"),
    }
}

pub fn optional(name: &str, default: &str) -> String {
    std::env::var(name).ok().filter(|value| !value.trim().is_empty()).unwrap_or_else(|| default.to_string())
}
