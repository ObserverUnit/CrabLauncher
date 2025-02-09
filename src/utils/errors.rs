use std::io;

use super::download;

#[derive(Debug)]
pub enum InstallationError {
    ZipError(zip::result::ZipError),
    DownloadError(download::DownloadError),
    IoError(io::Error),
}

impl From<zip::result::ZipError> for InstallationError {
    fn from(err: zip::result::ZipError) -> Self {
        InstallationError::ZipError(err)
    }
}
impl From<download::DownloadError> for InstallationError {
    fn from(err: download::DownloadError) -> Self {
        InstallationError::DownloadError(err)
    }
}

impl From<io::Error> for InstallationError {
    fn from(err: io::Error) -> Self {
        InstallationError::IoError(err)
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    InstallationError(InstallationError),
    MinecraftError(String),
}

impl From<InstallationError> for ExecutionError {
    fn from(err: InstallationError) -> Self {
        ExecutionError::InstallationError(err)
    }
}
