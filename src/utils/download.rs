use std::io::{self};

#[allow(dead_code)]
#[derive(Debug)]
pub enum DownloadError {
    InvaildUrl,
    Timeout,
    Other(reqwest::Error),
    Status(reqwest::StatusCode),
    IoError(io::Error),
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

impl From<io::Error> for DownloadError {
    fn from(err: io::Error) -> Self {
        DownloadError::IoError(err)
    }
}

// TODO: make this async
/// Downloads a file from a given url and returns it as a byte vector
pub fn get(url: &str) -> Result<Vec<u8>, DownloadError> {
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(DownloadError::Status(response.status()));
    }
    // TODO: return Bytes instead of Vec<u8>
    let bytes = response.bytes()?;
    Ok(bytes.to_vec())
}
