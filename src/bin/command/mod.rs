use anyhow::Result;
use clap::Subcommand;
use mi_service::MiIOService;

mod action;
mod list;
mod prop;
mod spec;

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "List all devices")]
    List(list::Args),
    #[command(about = "Get/Set device properties")]
    Prop(prop::Args),
    #[command(about = "Device action")]
    Action(action::Args),
    #[command(about = "Get device spec")]
    Spec(spec::Args),
    #[command(external_subcommand)]
    External(Vec<String>),
}

impl Commands {
    pub async fn exec(&self, svc: MiIOService) -> Result<()> {
        match self {
            Commands::List(args) => args.exec(svc).await?,
            Commands::Prop(args) => args.exec().await?,
            Commands::Action(args) => args.exec().await?,
            Commands::Spec(args) => args.exec(svc).await?,
            Commands::External(_args) => {
                //     if args.is_empty() {
                //         Cli::command().print_help().unwrap();
                //         return Ok(());
                //     }

                //     match std::env::var("MI_DID").ok() {
                //         Some(did) => {
                //             let cmd = &args[0];
                //             let args = &args[1..];
                //             let argc = args.len();
                //             debug!("did: {did} cmd: {cmd:?} args: {args:?}");
                //             let mut props: Vec<String> = Vec::new();
                //             let mut step = true;
                //             let miot = true;
                //             for item in cmd.split(',') {
                //                 let (key, value) = item.split_once('=').unwrap_or((item, ""));
                //                 let (siid, iid) = key.split_once('.').unwrap_or((key, "1"));
                //                 let mut prop;
                //                 if siid.chars().all(char::is_numeric)
                //                     && iid.chars().all(char::is_numeric)
                //                 {
                //                     prop =
                //                         vec![siid.parse::<i32>().unwrap(), iid.parse::<i32>().unwrap()];
                //                 } else {
                //                     step = false;
                //                     break;
                //                 }
                //             }
                //         }
                //         None => Cli::command().print_help().unwrap(),
                //     }
            }
        }
        Ok(())
    }
}
