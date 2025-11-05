use std::sync::Arc;

use crate::metrics::Metrics;
use crate::repositories::{
    EmailVerificationRepository, EmailVerificationRepositoryTrait, PasswordResetRepository,
    PasswordResetRepositoryTrait, RefreshTokenRepository, RefreshTokenRepositoryTrait,
    UserRepository, UserRepositoryTrait,
};
use crate::services::EmailService;
use axum::extract::FromRef;
use sqlx::PgPool;
use tracing::{error, info};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: PgPool,
    pub user_repository: Arc<dyn UserRepositoryTrait>,
    pub email_verification_repository: Arc<dyn EmailVerificationRepositoryTrait>,
    pub password_reset_repository: Arc<dyn PasswordResetRepositoryTrait>,
    pub refresh_token_repository: Arc<dyn RefreshTokenRepositoryTrait>,
    pub email_service: Arc<EmailService>,
    pub metrics: Option<Metrics>,
}

impl AppState {
    pub async fn new(database_url: &str, metrics: Option<Metrics>) -> Result<Self, sqlx::Error> {
        // Create the database connection pool
        let db = PgPool::connect(database_url).await?;

        sqlx::migrate!("./migrations").run(&db).await?;

        let user_repository: Arc<dyn UserRepositoryTrait> =
            Arc::new(UserRepository::new(db.clone()));

        let email_verification_repository: Arc<dyn EmailVerificationRepositoryTrait> =
            Arc::new(EmailVerificationRepository::new(db.clone()));

        let password_reset_repository: Arc<dyn PasswordResetRepositoryTrait> =
            Arc::new(PasswordResetRepository::new(db.clone()));

        let refresh_token_repository: Arc<dyn RefreshTokenRepositoryTrait> =
            Arc::new(RefreshTokenRepository::new(db.clone()));

        info!("Initializing email service...");
        let email_service = match EmailService::new() {
            Ok(service) => Arc::new(service),
            Err(e) => {
                error!("Failed to initialize email service: {}", e);
                error!("Make sure all SMTP env vars are set in .env");
                panic!("Email service initialization failed");
            }
        };

        Ok(Self {
            db,
            user_repository,
            email_verification_repository,
            password_reset_repository,
            refresh_token_repository,
            email_service,
            metrics,
        })
    }
}
