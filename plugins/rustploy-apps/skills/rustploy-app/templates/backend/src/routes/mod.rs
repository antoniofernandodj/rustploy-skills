//! Camada HTTP: só traduz requisição → service → resposta. Sem SQL e sem regra de negócio aqui.
mod auth;
mod health;

use axum::{routing::{get, post}, Router};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Todas as rotas ficam sob `/api` (o nginx do frontend repassa `/api/` para cá).
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health::health))
        .route("/auth/login", post(auth::login))
        .route("/auth/me", get(auth::me));

    Router::new().nest("/api", api).layer(TraceLayer::new_for_http()).with_state(state)
}
