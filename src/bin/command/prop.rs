use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
pub struct Args {
    op: Operation,
}

#[derive(Clone, Debug, ValueEnum)]
enum Operation {
    Get,
    Set,
}

impl Args {
    pub async fn exec(&self) -> anyhow::Result<()> {
        match self.op {
            Operation::Get => {
                println!("get");
            }
            Operation::Set => {
                println!("set");
            }
        }
        Ok(())
    }
}
