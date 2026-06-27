use crate::models::{providers, providers_llm};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: i32,
    pub temperature: f64,
}

pub async fn active_llm_providers(
    db: &DatabaseConnection,
) -> Result<HashMap<String, LlmConfig>, DbErr> {
    let rows = providers::Entity::find()
        .filter(providers::Column::ProviderType.eq("llm"))
        .filter(providers::Column::IsActive.eq(true))
        .find_with_related(providers_llm::Entity)
        .all(db)
        .await?;

    let mut map = HashMap::new();
    for (p, llms) in rows {
        for llm in llms {
            map.insert(
                p.name.clone(),
                LlmConfig {
                    name: p.name.clone(),
                    base_url: p.base_url.clone(),
                    api_key: p.api_key.clone(),
                    model: llm.model_name,
                    max_tokens: llm.max_tokens,
                    temperature: llm.temperature as f64,
                },
            );
        }
    }
    Ok(map)
}
