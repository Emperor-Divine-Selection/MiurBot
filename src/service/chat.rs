use crate::config;
use reqwest::Client;

pub async fn handle_chat(message: String) -> Result<String, String> {
    let api_key = &config::CONFIG.openai_api_key;
    let base_url = &config::CONFIG.openai_base_url;
    let model = &config::CONFIG.model;
    let client = Client::new();

    let body = serde_json::json!({
      "model": model,
      "messages": [
        {
          "role": "system",
          "content":  "你是 Miur，一个友好的 AI 助手。"
        },
        {
          "role": "user",
          "content": message
        }
      ]
    });
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
    Ok(reply.to_string())
}
