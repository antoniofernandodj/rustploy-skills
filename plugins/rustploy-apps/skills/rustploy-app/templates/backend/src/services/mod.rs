//! Regras de negócio. Não conhece HTTP nem SQL: recebe dados, usa repositories, devolve resultado ou AppError.
mod auth_service;
pub use auth_service::AuthService;
