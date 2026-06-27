mod adapter;
mod config;
mod models;
mod service;
mod store;

#[tokio::main]
async fn main() {
    println!("DataBase:{}", config::CONFIG.database_url);
    println!(
        "Server: {}:{}:{}",
        config::CONFIG.server_host,
        config::CONFIG.server_port,
        config::CONFIG.jwt_secret,
    );
    let _db = store::init().await.expect("链接数据库失败了");
    println!("连接数据库成功");
    adapter::axum::start().await;
}
