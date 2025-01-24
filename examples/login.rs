use mi_service::{init_tracing_subscriber, Account, TokenStore, MIIO_SID};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let level = dotenvy::var("LOG_LEVEL")?;
    init_tracing_subscriber(level.parse().ok());

    let username = dotenvy::var("MI_USER")?;
    let password = dotenvy::var("MI_PASS")?;
    let token_path = dotenvy::var("MI_TOKEN")?;

    let token_store = TokenStore::new(token_path).await;
    let mut account_svc = Account::new(username, password, token_store);
    if account_svc.login(MIIO_SID).await {
        println!("==");
    }
    Ok(())
}
