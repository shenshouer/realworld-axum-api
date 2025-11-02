use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

impl RefreshToken {
    // Check if token has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    // Check if token is valid (not expired AND not used)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_used
    }
}
