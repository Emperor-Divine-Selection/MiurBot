use crate::config;
use crate::models;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};
use std::collections::HashMap;
use std::sync::OnceLock;

pub mod provider;
pub mod session;
pub mod user;

pub static PROVIDERS: OnceLock<HashMap<String, provider::LlmConfig>> = OnceLock::new();

pub async fn init() -> Result<DatabaseConnection, DbErr> {
    let url = format!("sqlite://{}?mode=rwc", config::CONFIG.database_url);
    let db = Database::connect(&url).await?;
    create_tables(&db).await?;
    if !user::has_any_user(&db).await? {
        user::create_default_user(&db).await?;
        println!("默认用户已经创建")
    }
    let providers = provider::active_llm_providers(&db).await?;
    PROVIDERS
        .set(providers)
        .map_err(|_| DbErr::Custom("providers 已初始化".to_owned()))?;
    Ok(db)
}

async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    let schema = Schema::new(db.get_database_backend());

    let mut stmt = schema.create_table_from_entity(models::users::Entity);
    stmt.if_not_exists();
    db.execute(&stmt).await?;

    let mut stmt = schema.create_table_from_entity(models::sessions::Entity);
    stmt.if_not_exists();
    db.execute(&stmt).await?;

    let mut stmt = schema.create_table_from_entity(models::session_messages::Entity);
    stmt.if_not_exists();
    db.execute(&stmt).await?;

    let mut stmt = schema.create_table_from_entity(models::providers::Entity);
    stmt.if_not_exists();
    db.execute(&stmt).await?;

    let mut stmt = schema.create_table_from_entity(models::providers_llm::Entity);
    stmt.if_not_exists();
    db.execute(&stmt).await?;

    println!("5 张表就绪");
    Ok(())
}
