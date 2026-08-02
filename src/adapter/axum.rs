use crate::config;
use crate::service::chat::{self, ChatEvent};
use axum::{
    Extension, Json, Router,
    response::sse::{Event, Sse},
    routing::{get, post},
};
use futures_util::{Stream, StreamExt};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
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
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = match chat::handle_chat(&db, request.message).await {
        Ok(rx) => rx,
        Err(e) => {
            // 出错也走管道，发给前端
            let (tx, rx) = mpsc::channel::<ChatEvent>(1);
            let _ = tx.try_send(ChatEvent::Delta(format!("错误：{e}")));
            rx
        }
    };

    let stream = ReceiverStream::new(rx).map(|event| match event {
        ChatEvent::Delta(text) => Ok(Event::default().data(text)),
        ChatEvent::Done => Ok(Event::default().event("done").data("")),
    });
    Sse::new(stream)
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
