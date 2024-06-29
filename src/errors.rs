#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("RadioBrowserError: {0}")]
    RadioBrowserError(#[from] radiobrowser::RbError),
    #[error("ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("HttptError: {0}")]
    HttpError(reqwest::StatusCode),
    #[error("LockError: {0}")]
    LockError(String),
}
