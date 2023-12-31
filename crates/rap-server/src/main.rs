extern crate core;

mod config;
mod crypto;
mod inbox;
mod key;
mod signature;
mod signed;
mod users;
mod utils;
mod webfinger;

use crate::config::Config;
use crate::users::InMemoryPeopleStore;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::post;
use axum::{middleware, response::Json, routing::get, Extension, Router};
use axum_prometheus::PrometheusMetricLayerBuilder;
use clap::Parser;
use log::info;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceBuilder;

// `&'static str` becomes a `200 OK` with `content-type: text/plain; charset=utf-8`
async fn plain_text() -> &'static str {
    "boo!"
}

// `Json` gives a content-type of `application/json` and works with any type
// that implements `serde::Serialize`
async fn json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cfg = Config::parse();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix("rap_server")
        .with_default_metrics()
        .build_pair();

    // TODO: create a background task processor

    let people: Arc<dyn users::PeopleStore> = Arc::new(InMemoryPeopleStore::new());

    let app = Router::new()
        .route("/", get(plain_text))
        .route("/.well-known/webfinger", get(webfinger::json))
        .route("/users/:id", get(users::json))
        .route("/users/:id/inbox", post(inbox::json))
        .route("/plain_text", get(plain_text))
        .route("/json", get(json))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(request_logger))
                .layer(prometheus_layer)
                .layer(Extension(people))
                .layer(Extension(cfg.clone())),
        );

    let addr = format!("{}:{}", cfg.address, cfg.port);
    info!("Listening on {}", addr);

    let addr = addr.parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn request_logger<B>(
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let start = std::time::Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("");
    let response = next.run(request).await;
    // log request and response details here
    info!(
        "{} {}?{} ({} {}ms)",
        method,
        path,
        query,
        response.status(),
        start.elapsed().as_millis()
    );
    Ok(response)
}
