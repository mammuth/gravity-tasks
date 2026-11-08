mod api;
mod db;
mod lists;
mod tasks;

use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, Method};
use axum::response::Html;
use axum::routing::{any, get, patch, post};
use tower_http::cors::{AllowHeaders, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use crate::api::ApiError;

// Handlers lock one SQLite connection and run synchronously; queries take microseconds and
// a single-threaded runtime keeps the footprint minimal.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "storage/development.sqlite3".to_owned());
    let static_dir = PathBuf::from(env::var("STATIC_DIR").unwrap_or_else(|_| "public".to_owned()));
    let port: u16 = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);

    let conn = db::open(&db_path).unwrap_or_else(|e| {
        eprintln!("failed to open database {db_path}: {e}");
        std::process::exit(1);
    });

    // The built frontend, with an SPA fallback to index.html for client-side routes.
    let frontend = ServeDir::new(&static_dir).fallback(ServeFile::new(static_dir.join("index.html")));

    let app = Router::new()
        .route("/up", get(|| async { Html(r#"<!DOCTYPE html><html><body style="background-color: green"></body></html>"#) }))
        .route("/api/lists", get(lists::index).post(lists::create))
        .route("/api/lists/batch", post(lists::batch))
        .route("/api/lists/{id}", patch(lists::update).put(lists::update))
        .route("/api/tasks", get(tasks::index).post(tasks::create))
        .route("/api/tasks/batch", post(tasks::batch))
        .route("/api/tasks/{id}", patch(tasks::update).put(tasks::update))
        .route("/api/{*rest}", any(|| async { ApiError::NotFound }))
        .fallback_service(frontend)
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
        .layer(cors())
        .with_state(Arc::new(Mutex::new(conn)));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap_or_else(|e| {
        eprintln!("failed to bind port {port}: {e}");
        std::process::exit(1);
    });
    println!("gravity-tasks listening on :{port} (db: {db_path}, static: {})", static_dir.display());

    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await.expect("server error");
}

/// Allowed browser origins for cross-origin API calls (the Vite dev server by default).
fn cors() -> CorsLayer {
    let origins: Vec<HeaderValue> = env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173".to_owned())
        .split(',')
        .filter_map(|origin| origin.trim().parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_headers(AllowHeaders::mirror_request())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS, Method::HEAD])
}

async fn shutdown_signal() {
    let mut terminate =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("failed to install SIGTERM handler");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
}
