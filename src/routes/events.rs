use crate::models::{CreatePlanEvent, PlanEvent};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct EventFilter {
    pub start: Option<String>,
    pub end: Option<String>,
}

pub async fn list_events(
    State(pool): State<SqlitePool>,
    Query(opts): Query<EventFilter>,
) -> Result<Json<Vec<PlanEvent>>, StatusCode> {
    let query = if let (Some(start), Some(end)) = (opts.start, opts.end) {
        // filter by range: event overlaps with [start, end]
        // event.end_at >= start AND event.start_at <= end
        sqlx::query_as::<_, PlanEvent>(
            "SELECT * FROM plan_events WHERE end_at >= ? AND start_at <= ? ORDER BY start_at ASC",
        )
        .bind(start)
        .bind(end)
    } else {
        sqlx::query_as::<_, PlanEvent>("SELECT * FROM plan_events ORDER BY start_at ASC LIMIT 500")
    };

    match query.fetch_all(&pool).await {
        Ok(events) => Ok(Json(events)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_event(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreatePlanEvent>,
) -> Result<Json<PlanEvent>, StatusCode> {
    let result = sqlx::query_as::<_, PlanEvent>(
        r#"
        INSERT INTO plan_events (project_id, title, start_at, end_at, is_all_day, note)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(payload.project_id)
    .bind(payload.title)
    .bind(payload.start_at)
    .bind(payload.end_at)
    .bind(payload.is_all_day)
    .bind(payload.note)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(event) => Ok(Json(event)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_event(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM plan_events WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
