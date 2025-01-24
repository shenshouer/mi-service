use serde::{Deserialize, Serialize};

pub use miio::{MiIODevices, ResultData};
pub use mina::MiNaDevices;

mod miio;
mod mina;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T> {
    pub code: i32,
    #[serde(flatten)]
    pub data: T,
    pub message: String,
}
