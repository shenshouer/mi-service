use clap::Parser;
use mi_service::MiIOService;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    r#type: Option<String>,
    #[arg(short, long)]
    format: Option<String>,
}

impl Args {
    pub async fn exec(&self, svc: MiIOService) -> anyhow::Result<()> {
        println!("self: {:?}", self);
        let spec = svc
            .miot_spec(self.r#type.as_deref(), self.format.as_deref())
            .await?;
        let json_str = serde_json::to_string_pretty(&spec)?;
        println!("{json_str}");
        Ok(())
    }
}
