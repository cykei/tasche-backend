use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use crate::models::{Project, CreateProject};

pub async fn list_projects(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let result = sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY name ASC")
        .fetch_all(&pool)
        .await;

    match result {
        Ok(projects) => Ok(Json(projects)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_project(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateProject>,
) -> Result<Json<Project>, StatusCode> {
    let result = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (name, color, description)
        VALUES (?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(payload.name)
    .bind(payload.color)
    .bind(payload.description)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(project) => Ok(Json(project)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_project(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
