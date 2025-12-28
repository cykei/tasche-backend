use crate::models::{CreateTodo, Todo, UpdateTodo};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::collections::HashMap;

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
        Ok(mut list) => {
            attach_tags_to_todos(&pool, &mut list).await?;
            Ok(Json(list))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_todo(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut todo = sqlx::query_as::<_, Todo>(
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

    let normalized_tags = payload.tags.map(normalize_tags).unwrap_or_default();

    if !normalized_tags.is_empty() {
        link_tags(&mut tx, todo.id, &normalized_tags).await?;
    }

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    todo.tags = normalized_tags;

    Ok(Json(todo))
}

pub async fn update_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut todo = sqlx::query_as::<_, Todo>(
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
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.is_done)
    .bind(&payload.date)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let normalized_tags = payload.tags.map(normalize_tags);
    if let Some(ref tags) = normalized_tags {
        replace_tags(&mut tx, id, tags).await?;
    }

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(tags) = normalized_tags {
        todo.tags = tags;
    } else {
        let mut map = fetch_tags_map(&pool, &[todo.id]).await?;
        todo.tags = map.remove(&todo.id).unwrap_or_default();
    }

    Ok(Json(todo))
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

fn normalize_tags(input: Vec<String>) -> Vec<String> {
    input
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}

async fn link_tags(
    tx: &mut Transaction<'_, Sqlite>,
    todo_id: i64,
    tags: &[String],
) -> Result<(), StatusCode> {
    for tag_name in tags {
        let tag_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO tags (name) VALUES (?) ON CONFLICT(name) DO UPDATE SET name=name RETURNING id"
        )
        .bind(tag_name)
        .fetch_one(&mut *tx)
        .await
        .or_else(|_| {
            sqlx::query_scalar::<_, i64>("SELECT id FROM tags WHERE name = ?")
                .bind(tag_name)
                .fetch_one(&mut *tx)
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        sqlx::query("INSERT INTO todo_tags (todo_id, tag_id) VALUES (?, ?)")
            .bind(todo_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(())
}

async fn replace_tags(
    tx: &mut Transaction<'_, Sqlite>,
    todo_id: i64,
    tags: &[String],
) -> Result<(), StatusCode> {
    sqlx::query("DELETE FROM todo_tags WHERE todo_id = ?")
        .bind(todo_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !tags.is_empty() {
        link_tags(tx, todo_id, tags).await?;
    }

    Ok(())
}

async fn fetch_tags_map(
    pool: &SqlitePool,
    todo_ids: &[i64],
) -> Result<HashMap<i64, Vec<String>>, StatusCode> {
    if todo_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = vec!["?"; todo_ids.len()].join(",");
    let query = format!(
        "SELECT tt.todo_id as todo_id, tags.name as tag_name
         FROM todo_tags tt
         INNER JOIN tags ON tags.id = tt.tag_id
         WHERE tt.todo_id IN ({})",
        placeholders
    );

    let mut sql = sqlx::query(&query);
    for id in todo_ids {
        sql = sql.bind(id);
    }

    let rows = sql
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut map: HashMap<i64, Vec<String>> = HashMap::new();
    for row in rows {
        let todo_id: i64 = row.get("todo_id");
        let tag_name: String = row.get("tag_name");
        map.entry(todo_id).or_default().push(tag_name);
    }

    Ok(map)
}

async fn attach_tags_to_todos(pool: &SqlitePool, todos: &mut [Todo]) -> Result<(), StatusCode> {
    let ids: Vec<i64> = todos.iter().map(|todo| todo.id).collect();
    let mut tag_map = fetch_tags_map(pool, &ids).await?;
    for todo in todos.iter_mut() {
        if let Some(tags) = tag_map.remove(&todo.id) {
            todo.tags = tags;
        }
    }
    Ok(())
}
