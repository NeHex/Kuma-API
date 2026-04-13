mod common;
mod handlers;

use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handlers::root::hello))
        .route("/163music/{id}", get(handlers::music163::music_163))
        .route("/douban/{id}", get(handlers::douban::douban_subject))
        .route("/tmdb/{id}", get(handlers::tmdb::tmdb_movie))
        .fallback(handlers::root::hello);

    let addr = SocketAddr::from(([0, 0, 0, 0], 7788));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind 0.0.0.0:7788");

    println!("kuma-api listening on http://{}", addr);

    axum::serve(listener, app)
        .await
        .expect("server exited unexpectedly");
}
