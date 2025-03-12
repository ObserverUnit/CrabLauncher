use bytes::Bytes;
#[allow(dead_code)]
#[derive(Debug)]
pub enum DownloadError {
    InvaildUrl,
    Timeout,
    Other(reqwest::Error),
    Status(reqwest::StatusCode),
    Io(std::io::Error),
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            DownloadError::Timeout
        } else if err.is_builder() {
            DownloadError::InvaildUrl
        } else if let Some(status) = err.status() {
            DownloadError::Status(status)
        } else {
            DownloadError::Other(err)
        }
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(value: std::io::Error) -> Self {
        DownloadError::Io(value)
    }
}

/// Downloads a file from a given url and returns it as a byte vector
pub async fn get(url: &str) -> Result<Bytes, DownloadError> {
    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(DownloadError::Status(response.status()));
    }
    // TODO: return Bytes instead of Vec<u8>
    let bytes = response.bytes().await?;
    Ok(bytes)
}
