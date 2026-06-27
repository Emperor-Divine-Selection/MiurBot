use crate::{config, service::chat};
use axum::{
    Extension, Json, Router,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

async fn chat_handler(
    Extension(db): Extension<DatabaseConnection>,
    Json(request): Json<ChatRequest>,
) -> String {
    match chat::handle_chat(&db, request.message).await {
        Ok(reply) => reply,
        Err(e) => format!("错误：{}", e),
    }
}

pub async fn start(db: DatabaseConnection) {
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/chat", post(chat_handler))
        .layer(Extension(db));

    let addr = format!(
        "{}:{}",
        config::CONFIG.server_host,
        config::CONFIG.server_port
    );
    println!("服务器启动在 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
