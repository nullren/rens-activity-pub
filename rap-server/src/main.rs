use axum::{
    routing::get,
    response::Json,
    Router,
};
use clap::Parser;
use serde_json::{Value, json};

// `&'static str` becomes a `200 OK` with `content-type: text/plain; charset=utf-8`
async fn plain_text() -> &'static str {
    "foo"
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
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let app = Router::new()
        .route("/plain_text", get(plain_text))
        .route("/json", get(json));

    let addr = format!("{}:{}", cli.address, cli.port).parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
