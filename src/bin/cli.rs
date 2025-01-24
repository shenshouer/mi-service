use clap::{Parser, Subcommand};
use log::info;
use mi_service::{init_tracing_subscriber, Account, MiIOService, MiNaService, TokenStore};
use tracing::Level;

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
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "List all devices")]
    List {
        #[arg(long, help = "mina", default_value = "false")]
        mina: bool,
        #[arg(long, help = "是否是虚拟模式", default_value = "false")]
        get_virtual_model: Option<bool>,
        #[arg(long, help = "是否显示华米设备", default_value = "false")]
        get_huami_device: Option<bool>,
    },
    // Install {
    //     #[arg(help = "The name of the package to install")]
    //     package: String,
    // },
    // Remove {
    //     #[arg(help = "The name of the package to remove")]
    //     package: String,
    // },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    init_tracing_subscriber(cli.log_level);

    let token_store = TokenStore::new(cli.token_file.clone().unwrap().to_owned()).await;
    let account = Account::new(cli.user, cli.pass, token_store);

    match cli.command {
        Commands::List {
            mina,
            get_virtual_model,
            get_huami_device,
        } => {
            if mina {
                let mut mina_svc = MiNaService::new(account);
                let devices = mina_svc.devices(None).await?;
                info!("MiNADevices: {}", serde_json::to_string(&devices).unwrap());
            } else {
                let miio_svc = MiIOService::new(account, None);
                miio_svc
                    .devices(get_virtual_model, get_huami_device)
                    .await?;
            }
        }
    }
    Ok(())
}
