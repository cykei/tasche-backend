use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{Todo, CreateTodo, UpdateTodo};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FilterOptions {
    pub date: Option<String>,
}

pub async fn list_todos(
    State(pool): State<SqlitePool>,
    Query(opts): Query<FilterOptions>,
) -> Result<Json<Vec<Todo>>, StatusCode> {
    let todos = if let Some(date) = opts.date {
        sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE date = ? ORDER BY id DESC")
            .bind(date)
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as::<_, Todo>("SELECT * FROM todos ORDER BY date DESC, id DESC LIMIT 100")
            .fetch_all(&pool)
            .await
    };

    match todos {
        Ok(t) => Ok(Json(t)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_todo(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let mut tx = pool.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let todo = sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (title, content, is_done, date)
        VALUES (?, ?, 0, ?)
        RETURNING *
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.date)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        eprintln!("Failed to insert todo: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(tags) = payload.tags {
        for tag_name in tags {
            if tag_name.trim().is_empty() { continue; }
            
            // 1. Get or Create Tag
            // We use ON CONFLICT DO NOTHING if supported or selection logic. 
            // SQLite supports ON CONFLICT for unique constraints. Assuming name is unique.
            // First try verify/insert tag.
            
            let tag_id = sqlx::query_scalar::<_, i64>(
                "INSERT INTO tags (name) VALUES (?) ON CONFLICT(name) DO UPDATE SET name=name RETURNING id"
            )
            .bind(&tag_name)
            .fetch_one(&mut *tx)
            .await;

            // Fallback lookup if upsert fails or behaves oddly (though RETURNING work in newer sqlite)
            // If the above fails (e.g. older sqlite), we might need select. 
            // Let's assume standard upsert works or do a select content.
            
            let tid = match tag_id {
                Ok(id) => id,
                Err(_) => {
                    // Try finding it
                    sqlx::query_scalar::<_, i64>("SELECT id FROM tags WHERE name = ?")
                        .bind(&tag_name)
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                }
            };
            
            // 2. Link
            sqlx::query("INSERT INTO todo_tags (todo_id, tag_id) VALUES (?, ?)")
                .bind(todo.id)
                .bind(tid)
                .execute(&mut *tx)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(todo))
}

pub async fn update_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    // Dynamic update is tricky in pure SQL, so we fetch first or use COALESCE if fields are fixed.
    // Ideally we build the query dynamically or check exists.
    // For simplicity, let's just use separate queries or smart SQL.
    
    // Simple approach: Fetch, update struct, Save. Or just individual updates.
    // Let's use a COALESCE approach for atomic update
    
    let result = sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos
        SET 
            title = COALESCE(?, title),
            content = COALESCE(?, content),
            is_done = COALESCE(?, is_done),
            date = COALESCE(?, date),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(payload.title)
    .bind(payload.content)
    .bind(payload.is_done)
    .bind(payload.date)
    .bind(id)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(todo) => Ok(Json(todo)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
