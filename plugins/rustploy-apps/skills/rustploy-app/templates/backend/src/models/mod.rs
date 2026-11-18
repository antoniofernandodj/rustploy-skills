//! Entidades (o que vem do banco) e DTOs (o que entra e sai pela API).
mod user;
pub use user::{LoginRequest, LoginResponse, User, UserResponse};
