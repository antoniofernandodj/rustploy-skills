//! Camada de dados: o único lugar com SQL. Sempre com parâmetros ($1, $2 + bind).
mod user_repository;
pub use user_repository::UserRepository;
