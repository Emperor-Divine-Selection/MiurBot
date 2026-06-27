use crate::models::users;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, Set};

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
    let user = users::ActiveModel {
        username: Set("default".to_owned()),
        password_hash: Set("".to_owned()),
        ..Default::default()
    };
    let res = user.insert(db).await?;
    Ok(res.id)
}
