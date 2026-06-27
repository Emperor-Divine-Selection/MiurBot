use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, Default, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "session_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub session_id: i32,
    #[sea_orm(belongs_to, from = "session_id", to = "id")]
    pub session: HasOne<super::sessions::Entity>,
    pub role: String,
    pub content: String,
    pub created_at: DateTime,
}

impl ActiveModelBehavior for ActiveModel {}
