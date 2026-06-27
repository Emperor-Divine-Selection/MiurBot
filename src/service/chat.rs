use crate::{config, store};
use reqwest::Client;
use sea_orm::DatabaseConnection;

pub async fn handle_chat(db: &DatabaseConnection, message: String) -> Result<String, String> {
    let api_key = &config::CONFIG.openai_api_key;
    let base_url = &config::CONFIG.openai_base_url;
    let model = &config::CONFIG.model;
    let client = Client::new();
    let user_id = store::user::default_user_id(db)
        .await
        .map_err(|e| e.to_string())?;
    let session_id = store::session::get_or_create_session(db, user_id)
        .await
        .map_err(|e| e.to_string())?;

    // 查历史
    let history = store::session::recent_messages(db, session_id, 20)
        .await
        .map_err(|e| e.to_string())?;

    // 拼 messages 数组
    let mut messages =
        vec![serde_json::json!({"role":"system","content":"你是 Miur，一个友好的 AI 助手。"})];

    for msg in &history {
        messages.push(serde_json::json!({"role":&msg.role,"content":&msg.content}));
    }

    // 存用户消息
    store::session::save_message(db, session_id, "user", &message)
        .await
        .map_err(|e| e.to_string())?;

    messages.push(serde_json::json!({"role":"user","content":message}));

    let body = serde_json::json!({
      "model": model,
      "messages": messages
    });

    println!("发送给 API 的 messages: {:#?}", body);

    let resp = client
        .post(format!("{}/chat/completions", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let reply = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("");

    // 存 AI 回复
    store::session::save_message(db, session_id, "assistant", &reply)
        .await
        .map_err(|e| e.to_string())?;
    // 返回回复

    Ok(reply.to_string())
}
