//! Ponto de entrada. Subcomandos:
//!   serve             sobe a API (padrão do container)
//!   migrate-and-seed  aplica migrations e cria o usuário inicial (job `migrate`)
//!   migrate           só as migrations
mod config;
mod error;
mod models;
mod repositories;
mod routes;
mod security;
mod seed;
mod services;
mod state;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let command = std::env::args().nth(1).unwrap_or_else(|| "serve".into());
    let database_url = config::required("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&database_url)
        .await
        .context("não foi possível conectar ao banco (DATABASE_URL)")?;

    match command.as_str() {
        "migrate" => run_migrations(&pool).await,
        "migrate-and-seed" => {
            run_migrations(&pool).await?;
            seed::run(&pool, &seed::AdminSeed::from_env()?).await
        }
        "serve" => {
            // Também migra na subida: é idempotente e protegido por lock no banco,
            // e evita API sem tabelas se o job de pré-deploy não estiver ligado.
            run_migrations(&pool).await?;
            let config = config::Config::from_env()?;
            let bind_address = config.bind_address.clone();
            let app = routes::router(state::AppState::new(pool, config));
            let listener = tokio::net::TcpListener::bind(&bind_address).await?;
            tracing::info!(%bind_address, "API no ar");
            axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
            Ok(())
        }
        other => anyhow::bail!("subcomando desconhecido: {other} (use serve, migrate ou migrate-and-seed)"),
    }
}

async fn run_migrations(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await.context("falha ao aplicar migrations")?;
    tracing::info!("migrations aplicadas");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.ok(); };
    let terminate = async {
        if let Ok(mut signal) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            signal.recv().await;
        }
    };
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
