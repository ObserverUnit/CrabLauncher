use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::utils::{self, download::DownloadError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Download {
    pub id: Option<String>,
    pub path: Option<PathBuf>,
    pub sha1: String,
    pub size: i32,
    #[serde(rename = "totalSize")]
    pub total_size: Option<i32>,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub client: Download,
    pub client_mappings: Option<Download>,
    pub server: Download,
    pub server_mappings: Option<Download>,
}

impl Download {
    fn get(&self) -> Result<Vec<u8>, DownloadError> {
        utils::download::get(&self.url)
    }

    pub fn download_in(&self, path: &Path) -> Result<Vec<u8>, DownloadError> {
        let full_path = if let Some(ref child) = self.path {
            &path.join(child)
        } else {
            path
        };

        if full_path.exists() {
            return Ok(fs::read(full_path).unwrap());
        } else {
            let data = self.get()?;

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(full_path, &data)?;
            Ok(data)
        }
    }
}
