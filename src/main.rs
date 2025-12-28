use axum::{
    routing::get,
    Router,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tower_http::cors::CorsLayer;
use dotenvy::dotenv;
use std::env;

mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Setup database connection with create_if_missing
    let connection_options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .connect_with(connection_options)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    println!("Migrations applied successfully.");

    // Setup app
    let app = Router::new()
        .merge(routes::create_router())
        .route("/", get(root))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello from Rust Backend!"
}
