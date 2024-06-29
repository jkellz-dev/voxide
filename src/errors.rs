#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("RadioBrowserError: {0}")]
    RadioBrowser(#[from] radiobrowser::RbError),
    #[error("ReqwestError: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("HttptError: {0}")]
    Http(reqwest::StatusCode),
    #[error("LockError: {0}")]
    Lock(String),
}
