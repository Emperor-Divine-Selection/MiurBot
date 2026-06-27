use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, Default, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub user_id: i32,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: HasOne<super::users::Entity>,
    pub summary: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
impl ActiveModelBehavior for ActiveModel {}
