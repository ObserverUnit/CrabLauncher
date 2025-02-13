use serde::Deserialize;

use std::fs::{self};

use crate::{
    utils::{self, download::DownloadError},
    LAUNCHER_PATH,
};

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum VersionKind {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

#[derive(Deserialize, Debug)]
pub struct Version {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub versions: Vec<Version>,
}

impl Manifest {
    pub fn get() -> Self {
        let path = LAUNCHER_PATH.join("version_manifest.json");
        // download version info
        let res =
            reqwest::blocking::get("https://launchermeta.mojang.com/mc/game/version_manifest.json");
        // if offline use pre-downloaded file
        if let Ok(res) = res {
            let bytes = res.bytes().unwrap();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("failed creating dir");
            }
            fs::write(&path, bytes).expect("failed writing file version_manifest.json");
        }

        let buffer = fs::read_to_string(path).expect("failed reading file version_manifest.json");
        serde_json::from_str(buffer.as_str()).expect("failed parsing file version_manifest.json")
    }

    /// downloads client.json for a given version
    pub fn download_version(&self, version: &str) -> Result<Option<String>, DownloadError> {
        let Some(version) = self.versions.iter().find(|x| x.id == version) else {
            return Ok(None);
        };
        let res = utils::download::get(&version.url)?;
        Ok(Some(String::from_utf8(res).unwrap()))
    }
}
