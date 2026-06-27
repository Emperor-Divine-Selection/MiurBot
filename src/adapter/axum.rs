use crate::{config, service::chat};
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::Deserialize;

// 一个简单的处理函数
async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

async fn chat_handler(Json(request): Json<ChatRequest>) -> String {
    match chat::handle_chat(request.message).await {
        Ok(reply) => reply,
        Err(e) => format!("错误：{}", e),
    }
}

// 启动服务器
pub async fn start() {
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/chat", post(chat_handler));
    let addr = format!(
        "{}:{}",
        config::CONFIG.server_host,
        config::CONFIG.server_port
    );
    println!("服务器启动在 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
