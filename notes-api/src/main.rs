use std::{env, sync::Arc};

use axum::Router;

use crate::state::AppState;

pub mod api;
pub mod db;
pub mod extractors;
pub mod services;
pub mod state;
pub mod tokens;
pub mod utilities;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_state = Arc::new(AppState::init().await?);

    let app = Router::new().nest("/api", api::create_router(app_state));

    let ip = env::var("LISTENER_IP").unwrap_or("0.0.0.0".into());
    let port = env::var("LISTENER_PORT").unwrap_or("3123".into());
    let address = format!("{}:{}", ip, port);

    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
