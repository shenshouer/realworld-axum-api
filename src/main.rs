use axum::{
    Router,
    routing::{get, post},
};
use std::env;
use tower_http::trace::TraceLayer;
use tracing::info;

use realworld_axum_api::{
    errors::AppError,
    handlers::{
        current_user, forgot_password, health_check, login, logout, refresh_token, register,
        reset_password, verify_email,
    },
    otlp,
    state::AppState,
    views::{greeting_handler, index_handler, start_handler},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let endpoint = "http://localhost:5081";
    let logger_level = "info";
    otlp::init_tracing(logger_level, endpoint);

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file or environment");

    let app_state = AppState::new(&database_url)
        .await
        .expect("Failed to connect to database");

    info!("Connected to database successfully!");

    let app = Router::new()
        .route("/", get(start_handler))
        .route("/{lang}/index.html", get(index_handler))
        .route("/{lang}/greet-me.html", get(greeting_handler))
        .fallback(|| async { AppError::NotFound })
        .route("/health", get(health_check))
        .route("/api/users", post(register))
        .route("/api/users/login", post(login))
        .route("/api/user", get(current_user))
        .route("/api/auth/verify-email", get(verify_email))
        .route("/api/auth/forgot-password", post(forgot_password))
        .route("/api/auth/reset-password", post(reset_password))
        .route("/api/auth/refresh", post(refresh_token))
        .route("/api/auth/logout", post(logout))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(Error::Bind)?;
    if let Ok(addr) = listener.local_addr() {
        info!("Server running on http://{addr}/");
    }

    axum::serve(listener, app).await.map_err(Error::Run)
}

#[derive(displaydoc::Display, pretty_error_debug::Debug, thiserror::Error)]
enum Error {
    /// could not bind socket
    Bind(#[source] std::io::Error),
    /// could not run server
    Run(#[source] std::io::Error),
}
