/// Represents all possible errors that can occur in the application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error returned from the RadioBrowser API.
    #[error("RadioBrowserError: {0}")]
    RadioBrowser(#[from] radiobrowser::RbError),
    /// Error returned from the Reqwest HTTP client.
    #[error("ReqwestError: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// HTTP error with a specific status code.
    #[error("HttptError: {0}")]
    Http(reqwest::StatusCode),
    /// Error related to locking resources.
    #[error("LockError: {0}")]
    Lock(String),
}
