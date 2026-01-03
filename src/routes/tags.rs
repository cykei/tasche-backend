use axum::{extract::State, http::StatusCode, Json};
use sqlx::SqlitePool;

pub async fn list_tags(State(pool): State<SqlitePool>) -> Result<Json<Vec<String>>, StatusCode> {
    let tags = sqlx::query_scalar::<_, String>("SELECT name FROM tags ORDER BY name ASC")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tags))
}
