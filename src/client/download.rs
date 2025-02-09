use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use super::Index;
use crate::{error::Error, ASSETS_PATH};
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
    fn get(&self) -> Result<reqwest::blocking::Response, Error> {
        reqwest::blocking::get(&self.url).map_err(|e| Error::Download(format!("{e}")))
    }

    pub fn download_in(&self, path: &Path) -> Result<Vec<u8>, Error> {
        let full_path = if let Some(ref child) = self.path {
            &path.join(child)
        } else {
            path
        };

        if full_path.exists() {
            return Ok(fs::read(full_path).unwrap());
        } else {
            let mut data = self.get()?;
            let mut buf = Vec::with_capacity(self.size as usize);
            data.read_to_end(&mut buf).unwrap();

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(full_path, &buf).unwrap();
            Ok(buf)
        }
    }

    pub fn download_indexs(&self) -> Result<(), Error> {
        if let Some(ref id) = self.id {
            let indexes_dir = ASSETS_PATH.join("indexes");
            let indexes_path = indexes_dir.join(format!("{}.json", id));
            let download = self.download_in(&indexes_path)?;

            // downloading objects
            let index: Index = serde_json::from_slice(&download).unwrap();
            let objects = index.objects;

            for (_, object) in objects {
                let dir_name = &object.hash[0..2];
                let dir = ASSETS_PATH.join("objects").join(dir_name);
                let path = dir.join(&object.hash);

                if path.exists() {
                    continue;
                }

                fs::create_dir_all(&dir).unwrap();
                let res = reqwest::blocking::get(format!(
                    "https://resources.download.minecraft.net/{dir_name}/{}",
                    object.hash
                ))
                .unwrap();

                fs::write(path, res.bytes().unwrap()).unwrap();
            }
            Ok(())
        } else {
            todo!()
        }
    }
}
