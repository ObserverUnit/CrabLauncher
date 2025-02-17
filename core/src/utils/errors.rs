use std::io;

use super::download::DownloadError;

#[derive(Debug)]
pub enum CoreError<'a> {
    ZipError(zip::result::ZipError),
    DownloadError(DownloadError),
    IoError(io::Error),
    MinecraftVersionNotFound,
    ProfileNotFound(&'a str),
    MinecraftFailure(i32),
}

impl From<DownloadError> for CoreError<'static> {
    fn from(value: DownloadError) -> Self {
        Self::DownloadError(value)
    }
}

impl From<std::io::Error> for CoreError<'static> {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
impl From<zip::result::ZipError> for CoreError<'static> {
    fn from(value: zip::result::ZipError) -> Self {
        Self::ZipError(value)
    }
}
