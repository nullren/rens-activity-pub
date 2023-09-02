mod users;
mod webfinger;

use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::{middleware, response::Json, routing::get, Router};
use axum_prometheus::PrometheusMetricLayerBuilder;
use clap::Parser;
use serde_json::{json, Value};
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Address to listen on
    #[arg(short, long, env, default_value = "0.0.0.0")]
    address: String,

    /// Port to listen on
    #[arg(short, long, env, default_value = "3000")]
    port: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix("rap_server")
        .with_default_metrics()
        .build_pair();

    let app = Router::new()
        .route("/", get(plain_text))
        .route("/.well-known/webfinger", get(webfinger::json))
        .route("/.well-known/host-meta", get(json))
        .route("/users/:id", get(users::json))
        .route("/users/:id/inbox", get(json))
        .route("/plain_text", get(plain_text))
        .route("/json", get(json))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(
            ServiceBuilder::new()
                .layer(prometheus_layer)
                .layer(middleware::from_fn(request_logger)),
        );

    let addr = format!("{}:{}", cli.address, cli.port);
    log::info!("Listening on {}", addr);

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
    let path = uri.path().clone();
    let query = uri.query().unwrap_or("");
    let response = next.run(request).await;
    // log request and response details here
    log::info!(
        "{} {}?{} ({} {}ms)",
        method,
        path,
        query,
        response.status(),
        start.elapsed().as_millis()
    );
    Ok(response)
}
