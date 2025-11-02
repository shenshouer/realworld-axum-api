use axum::{extract::State, response::Json};
use serde_json::{Value, json};
use tracing::error;

use crate::state::AppState;

pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => Json(json!({
            "status": "ok",
            "message": "Server is running"
        })),
        Err(e) => {
            error!("Database  error: {e}");
            Json(json!({
                "status": "error",
                "message": "disconnected",
                "error": e.to_string()
            }))
        }
    }
}
