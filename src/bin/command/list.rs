use clap::Parser;
use log::info;
use mi_service::MiIOService;

/// list 子命令
#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, help = "设备名称")]
    name: Option<String>,
    #[arg(long, help = "是否是虚拟模式", default_value = "false")]
    get_virtual_model: Option<bool>,
    #[arg(long, help = "是否显示华米设备", default_value = "false")]
    get_huami_device: Option<bool>,
}

impl Args {
    pub async fn exec(&self, svc: MiIOService) -> anyhow::Result<()> {
        let res = svc
            .devices(
                self.name.as_deref(),
                self.get_virtual_model,
                self.get_huami_device,
            )
            .await?;
        info!(
            "MiIODevices: {}",
            serde_json::to_string_pretty(&res).unwrap()
        );
        Ok(())
    }
}
