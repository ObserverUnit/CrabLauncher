use std::{
    fs::{self},
    path::Path,
};

use bytes::Bytes;
use crab_launcher_api::meta::manifest::{Version, VersionManifest};

use crate::utils::{self, errors::CoreError};

/// parses the global version manifest
async fn fetch_global_manifest(launcher_root: &Path) -> VersionManifest {
    let path = launcher_root.join("version_manifest.json");
    // download version info
    let res =
        utils::download::get("https://launchermeta.mojang.com/mc/game/version_manifest.json").await;
    // if offline use pre-downloaded file
    if let Ok(res) = res {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed creating dir");
        }
        fs::write(&path, res).expect("failed writing file version_manifest.json");
    }

    let buffer = fs::read_to_string(path).expect("failed reading file version_manifest.json");
    serde_json::from_str(buffer.as_str()).expect("failed parsing file version_manifest.json")
}

#[derive(Debug)]
pub struct Manifest {
    inner: VersionManifest,
}

impl Manifest {
    pub async fn fetch(launcher_root: &Path) -> Self {
        let inner = fetch_global_manifest(launcher_root).await;
        Self { inner }
    }

    pub fn versions(&self) -> impl Iterator<Item = &Version> {
        self.inner.versions.iter()
    }

    /// downloads client.json for a given minecraft version and the client.json contents as a string
    pub async fn download_version(&self, version: &str) -> Result<Bytes, CoreError<'static>> {
        let Some(version) = self.versions().find(|x| x.id == version) else {
            return Err(CoreError::MinecraftVersionNotFound);
        };
        let res = utils::download::get(&version.url).await?;
        Ok(res)
    }
}
