use axum::{
    Router,
    routing::get,
    response::Json
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use std::net::SocketAddr;
use serde_json::{Value, json};

//routes
async fn root()->&'static str{
    "hello world!"
}

async fn json()->Json<Value>{
    Json(json!({"message":"Hello world!"}))
}

#[tokio::main]
async fn main(){
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app=Router::new()
        .route("/",get(root))
        .route("/json",get(json))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    trace::DefaultMakeSpan::new()
                        .level(Level::INFO)
                )
                .on_response(
                    trace::DefaultOnResponse::new().level(Level::INFO)
                ),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
