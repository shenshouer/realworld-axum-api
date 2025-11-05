use axum::{
    BoxError, Router,
    error_handling::HandleErrorLayer,
    http::{
        Method, Request,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    routing::{get, post},
};
use opentelemetry::global;
use std::{env, time::Duration};
use tower::{ServiceBuilder, timeout::TimeoutLayer};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer, trace::TraceLayer,
};
use tracing::info;

use realworld_axum_api::{
    auth::middleware::track_metrics,
    errors::AppError,
    handlers::{
        current_user, forgot_password, health_check, login, logout, refresh_token, register,
        reset_password, verify_email,
    },
    metrics::Metrics,
    otlp,
    state::AppState,
    views::{greeting_handler, index_handler, start_handler},
};

async fn handle_timeout_error(err: BoxError) -> (axum::http::StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            axum::http::StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let endpoint = env::var("OLTP_ENDPOINT").ok();
    let token = env::var("OLTP_TOKEN").ok();

    let logger_level = Some("info".to_owned());
    let meter_provider = otlp::init_tracing(logger_level, endpoint, token).unwrap();
    let meter = meter_provider.as_ref().map(|_| global::meter("my_meter"));
    let metrics = meter.map(Metrics::new);

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file or environment");

    let app_state = AppState::new(&database_url, metrics)
        .await
        .expect("Failed to connect to database");

    info!("Connected to database successfully!");

    // 跨域
    let cors = CorsLayer::new()
        .allow_origin(
            "https://example.com"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![CONTENT_TYPE])
        .expose_headers(vec![CONTENT_TYPE]);
    // 压缩头部
    // let predicate = DefaultPredicate::new()
    //     .and(NotForContentType::new("application/json"));
    let compression = CompressionLayer::new()
        .gzip(true) // 启用 Gzip
        .br(true); // 启用 Brotli（需 feature "compression-br"）

    // 屏蔽日志中 Token 敏感信息， 需要sensitive-headers支持
    let sensitive = SetSensitiveRequestHeadersLayer::new(vec![AUTHORIZATION]);
    let trace = TraceLayer::new_for_http().make_span_with(
        |request: &Request<_>| {
            tracing::info_span!("http_req", method = %request.method(), uri = %request.uri())
        },
    );
    let timeout = TimeoutLayer::new(Duration::from_secs(30));
    let app = Router::new()
        .route("/", get(start_handler))
        .route("/{lang}/index.html", get(index_handler))
        .route("/{lang}/greet-me.html", get(greeting_handler))
        .route("/health", get(health_check))
        .route("/api/users", post(register))
        .route("/api/users/login", post(login))
        .route("/api/user", get(current_user))
        .route("/api/auth/verify-email", get(verify_email))
        .route("/api/auth/forgot-password", post(forgot_password))
        .route("/api/auth/reset-password", post(reset_password))
        .route("/api/auth/refresh", post(refresh_token))
        .route("/api/auth/logout", post(logout))
        .fallback(|| async { AppError::NotFound })
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            track_metrics,
        ))
        .with_state(app_state)
        .layer(cors)
        .layer(compression)
        .layer(sensitive)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .layer(timeout),
        )
        .layer(trace);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(Error::Bind)?;
    if let Ok(addr) = listener.local_addr() {
        info!("Server running on http://{addr}/");
    }

    axum::serve(listener, app).await.map_err(Error::Run)?;
    if let Some(meter_provider) = meter_provider {
        meter_provider.shutdown().map_err(Error::OTel)?;
    }
    Ok(())
}

#[derive(displaydoc::Display, pretty_error_debug::Debug, thiserror::Error)]
enum Error {
    /// could not bind socket
    Bind(#[source] std::io::Error),
    /// could not run server
    Run(#[source] std::io::Error),
    /// could not shutdown meter provider
    OTel(#[source] opentelemetry_sdk::error::OTelSdkError),
}
