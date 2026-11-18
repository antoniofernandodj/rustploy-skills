use axum::{extract::State, Json};

use crate::{
    error::AppResult,
    models::{LoginRequest, LoginResponse, UserResponse},
    security::AuthUser,
    state::AppState,
};

pub async fn login(State(state): State<AppState>, Json(request): Json<LoginRequest>) -> AppResult<Json<LoginResponse>> {
    Ok(Json(state.auth_service.login(request).await?))
}

pub async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<UserResponse>> {
    Ok(Json(state.auth_service.current_user(user.id).await?.into()))
}
