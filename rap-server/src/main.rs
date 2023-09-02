use axum::{
    routing::get,
    response::Json,
    Router,
};
use axum_prometheus::{PrometheusMetricLayer, PrometheusMetricLayerBuilder};
use clap::Parser;
use serde_json::{Value, json};

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
    let cli = Cli::parse();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix("rap_server")
        .build_pair();

    let app = Router::new()
        .route("/", get(plain_text))
        .route("/plain_text", get(plain_text))
        .route("/json", get(json))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer);

    let addr = format!("{}:{}", cli.address, cli.port);
    println!("Listening on {}", addr);

    let addr = addr.parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
