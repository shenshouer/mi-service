use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    pub did: String,
}

impl Args {
    pub async fn exec(&self) -> anyhow::Result<()> {
        println!("spec: {:?}", self);
        Ok(())
    }
}
