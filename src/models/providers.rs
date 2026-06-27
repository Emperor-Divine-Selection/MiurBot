use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, Default, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "providers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub api_key: String,
    #[sea_orm(default_value = "1")]
    pub is_active: bool,
    pub created_at: DateTime,
    #[sea_orm(has_many)]
    pub llm_configs: HasMany<super::providers_llm::Entity>,
}
impl ActiveModelBehavior for ActiveModel {}
