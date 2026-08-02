use crate::models::users;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};

// 检查是否存在任何用户
pub async fn has_any_user(db: &DatabaseConnection) -> Result<bool, DbErr> {
    let count = users::Entity::find().count(db).await?;
    match count {
        0 => Ok(false),
        _ => Ok(true),
    }
}

// 创建默认用户
pub async fn create_default_user(db: &DatabaseConnection) -> Result<i32, DbErr> {
    let hash =
        bcrypt::hash("123456", bcrypt::DEFAULT_COST).map_err(|e| DbErr::Custom(e.to_string()))?;
    let user = users::ActiveModel {
        username: Set("default".to_owned()),
        password_hash: Set(hash),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    let res = user.insert(db).await?;
    Ok(res.id)
}

// 获取默认用户 id 的函数
pub async fn default_user_id(db: &DatabaseConnection) -> Result<i32, DbErr> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq("default"))
        .one(db)
        .await?;
    Ok(user.unwrap().id)
}

pub async fn find_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(db)
        .await
}
