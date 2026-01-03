use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;

pub async fn list_tags(State(pool): State<SqlitePool>) -> Result<Json<Vec<String>>, StatusCode> {
    let tags = sqlx::query_scalar::<_, String>("SELECT name FROM tags ORDER BY name ASC")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tags))
}

pub async fn delete_tag(
    State(pool): State<SqlitePool>,
    Path(name): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let normalized = name.trim();
    if normalized.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tag_id: Option<i64> = sqlx::query_scalar("SELECT id FROM tags WHERE name = ?")
        .bind(normalized)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(tag_id) = tag_id {
        sqlx::query("DELETE FROM todo_tags WHERE tag_id = ?")
            .bind(tag_id)
            .execute(tx.as_mut())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(tag_id)
            .execute(tx.as_mut())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
