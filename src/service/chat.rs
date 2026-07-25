use crate::{config, store};
use futures_util::StreamExt;
use reqwest::Client;
use sea_orm::DatabaseConnection;

pub async fn handle_chat(db: &DatabaseConnection, message: String) -> Result<String, String> {
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

    let resp = call_llm_stream(messages).await?;
    store::session::save_message(db, session_id, "assistant", &resp)
        .await
        .map_err(|e| e.to_string())?;
    Ok(resp)
}

async fn call_llm_stream(messages: Vec<serde_json::Value>) -> Result<String, String> {
    let body = serde_json::json!({
      "model": &config::CONFIG.model,
      "messages":messages,
      "stream":true
    });

    let resp = Client::new()
        .post(format!(
            "{}/chat/completions",
            &config::CONFIG.openai_base_url
        ))
        .header(
            "Authorization",
            format!("Bearer {}", &config::CONFIG.openai_api_key),
        )
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    let mut reply = String::new();

    while let Some(chunk) = stream.next().await {
        // 从 Result 中提取出字节块（Bytes），出错就转成 String 抛出
        let chunk = chunk.map_err(|e| e.to_string())?;
        buf.extend_from_slice(&chunk);

        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&line_bytes);
            let line = line.trim();
            if let Some(text) = parse_sse_line(line) {
                print!("{}", text);
                reply.push_str(&text);
            }
        }
    }
    println!();
    Ok(reply)
}

fn parse_sse_line(line: &str) -> Option<String> {
    let data = line.trim().strip_prefix("data:")?.trim();
    if data == "[DONE]" {
        return None;
    }
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    let text = json["choices"][0]["delta"]["content"].as_str()?;
    Some(text.to_string())
}
