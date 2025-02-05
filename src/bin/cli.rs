use clap::{CommandFactory, Parser};
use command::Commands;
use mi_service::{init_tracing_subscriber, Account, MiIOService, TokenStore};
use tracing::Level;

mod command;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(
        short,
        long,
        help = "日志级别",
        env = "LOG_LEVEL",
        default_value = "info"
    )]
    log_level: Option<Level>,
    #[arg(short, long, help = "Username账号", env = "MI_USER")]
    user: String,
    #[arg(short, long, help = "Password密码", env = "MI_PASS")]
    pass: String,
    #[arg(
        short,
        long,
        help = "Token文件路径",
        env = "MI_TOKEN",
        default_value = "~/.mi.token"
    )]
    token_file: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    init_tracing_subscriber(cli.log_level);

    let token_store = TokenStore::new(cli.token_file.clone().unwrap().to_owned()).await;
    let account = Account::new(cli.user, cli.pass, token_store);

    let miio_svc = MiIOService::new(account, None);

    match cli.command {
        Some(cmd) => cmd.exec(miio_svc).await?,
        None => {
            Cli::command().print_help().unwrap();
        }
    }
    Ok(())
}
