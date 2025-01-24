use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub const MIIO_SID: &str = "xiaomiio";
pub const MINA_SID: &str = "micoapi";

pub use account::Account;
pub use miio::{MiIOService, SignData};
pub use mina::MiNaService;
pub use store::TokenStore;

mod account;
mod errors;
mod miio;
mod mina;
mod resp;
mod store;
mod utils;

pub fn init_tracing_subscriber(level: Option<Level>) {
    let level = level.unwrap_or(Level::INFO);

    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy()
                .add_directive("hyper=error".parse().unwrap())
                .add_directive("tower=error".parse().unwrap())
                .add_directive("h2=error".parse().unwrap())
                .add_directive("opentelemetry_sdk=error".parse().unwrap())
                .add_directive("sqlx=error".parse().unwrap())
                .add_directive("reqwest=error".parse().unwrap())
                .add_directive("cookie_store=error".parse().unwrap()),
        )
        .with(tracing_subscriber::fmt::layer().with_line_number(true))
        .init();
}
