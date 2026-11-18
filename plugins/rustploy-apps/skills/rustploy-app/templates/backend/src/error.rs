//! Erro único da aplicação. Cada variante vira um status HTTP; detalhes internos
//! vão para o log, nunca para o cliente.
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("dados inválidos: {0}")]
    Validation(String),
    #[error("não autenticado")]
    Unauthorized,
    #[error("sem permissão")]
    Forbidden,
    #[error("não encontrado")]
    NotFound,
    #[error("conflito: {0}")]
    Conflict(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Validation(message) => (StatusCode::BAD_REQUEST, message.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "não autenticado".into()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "sem permissão".into()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "não encontrado".into()),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message.clone()),
            AppError::Database(error) => {
                tracing::error!(%error, "erro de banco");
                (StatusCode::INTERNAL_SERVER_ERROR, "erro interno".into())
            }
            AppError::Internal(error) => {
                tracing::error!(error = %error, "erro interno");
                (StatusCode::INTERNAL_SERVER_ERROR, "erro interno".into())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
