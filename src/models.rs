use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub is_done: bool,
    pub date: String, // YYYY-MM-DD
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(default)]
    #[sqlx(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTodo {
    pub title: String,
    pub content: String,
    pub date: String,
    pub tags: Option<Vec<String>>, // List of tag names
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_done: Option<bool>,
    pub date: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PlanEvent {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub start_at: NaiveDateTime,
    pub end_at: NaiveDateTime,
    pub is_all_day: bool,
    pub note: Option<String>,
    pub linked_todo_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePlanEvent {
    pub project_id: i64,
    pub title: String,
    pub start_at: NaiveDateTime,
    pub end_at: NaiveDateTime,
    pub is_all_day: bool,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}
