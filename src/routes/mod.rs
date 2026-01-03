use axum::{
    routing::{delete, get, patch},
    Router,
};
use sqlx::SqlitePool;

mod events;
mod projects;
mod tags;
mod todos;

pub fn create_router() -> Router<SqlitePool> {
    Router::new()
        // Todos
        .route("/todos", get(todos::list_todos).post(todos::create_todo))
        .route(
            "/todos/:id",
            patch(todos::update_todo).delete(todos::delete_todo),
        )
        .route(
            "/todos/:id/tags/:tag",
            delete(todos::delete_todo_tag),
        )
        .route("/tags", get(tags::list_tags))
        // Projects
        .route(
            "/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route("/projects/:id", delete(projects::delete_project))
        // Events
        .route(
            "/events",
            get(events::list_events).post(events::create_event),
        )
        .route("/events/:id", delete(events::delete_event))
}
