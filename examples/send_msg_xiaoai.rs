use mi_service::{init_tracing_subscriber, Account, MiIOService, TokenStore};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let level = dotenvy::var("LOG_LEVEL")?;
    init_tracing_subscriber(level.parse().ok());

    let username = dotenvy::var("MI_USER")?;
    let password = dotenvy::var("MI_PASS")?;
    let token_path = dotenvy::var("MI_TOKEN")?;

    let token_store = TokenStore::new(token_path).await;
    let account = Account::new(username, password, token_store);
    let miio_svc = MiIOService::new(account, None);
    miio_svc
        .miot_action("102130584", (5, 1), Some(vec![json!("测试,哈哈哈, 你好")]))
        .await?;
    Ok(())
}
