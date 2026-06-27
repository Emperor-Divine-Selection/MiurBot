use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, Default, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime,
}
impl ActiveModelBehavior for ActiveModel {}
