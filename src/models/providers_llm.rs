use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Default, DeriveEntityModel)]
#[sea_orm(table_name = "providers_llm")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub provider_id: i32,
    #[sea_orm(belongs_to, from = "provider_id", to = "id")]
    pub provider: HasOne<super::providers::Entity>,
    pub model_name: String,
    #[sea_orm(default_value = "4096")]
    pub max_tokens: i32,
    #[sea_orm(default_value = "0.7")]
    pub temperature: f32,
}
impl ActiveModelBehavior for ActiveModel {}
