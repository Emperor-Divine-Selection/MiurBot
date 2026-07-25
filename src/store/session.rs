use crate::models::{session_messages, sessions};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

// 为默认用户创建新 session （或查找最近一个）
pub async fn get_or_create_session(db: &DatabaseConnection, user_id: i32) -> Result<i32, DbErr> {
    let session = sessions::Entity::find()
        .filter(sessions::Column::UserId.eq(user_id))
        .order_by_desc(sessions::Column::UpdatedAt)
        .one(db)
        .await?;
    match session {
        Some(s) => Ok(s.id),
        None => {
            let now = Utc::now().naive_utc();
            let new = sessions::ActiveModel {
                user_id: Set(user_id),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            Ok(new.insert(db).await?.id)
        }
    }
}

// 存一条消息
pub async fn save_message(
    db: &DatabaseConnection,
    session_id: i32,
    role: &str,
    content: &str,
) -> Result<(), DbErr> {
    let msg = session_messages::ActiveModel {
        session_id: Set(session_id),
        role: Set(role.to_owned()),
        content: Set(content.to_owned()),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    msg.insert(db).await?;
    Ok(())
}

// 获取最近 N 条消息（按时间升序，最早在前)
pub async fn recent_messages(
    db: &DatabaseConnection,
    session_id: i32,
    limit: u64,
) -> Result<Vec<session_messages::Model>, DbErr> {
    let mut messages = session_messages::Entity::find()
        .filter(session_messages::Column::SessionId.eq(session_id))
        .order_by_desc(session_messages::Column::CreatedAt)
        .limit(limit)
        .all(db)
        .await?;
    messages.reverse();
    Ok(messages)
}
