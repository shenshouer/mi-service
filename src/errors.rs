use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ServiceLogin Error: {0}")]
    ServiceLogin(String),
    #[error("SecurityTokenService Error: {0}")]
    SecurityTokenService(String),
    #[error("Request Error: {0}")]
    Request(String),
}
