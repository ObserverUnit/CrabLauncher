use std::fs::{self};

use crab_launcher_api::meta::manifest::{Version, VersionManifest};
use lazy_static::lazy_static;

use crate::{
    utils::{self, errors::CoreError},
    LAUNCHER_PATH,
};

/// parses the global version manifest
fn read_global_manifest() -> VersionManifest {
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

lazy_static! {
    static ref MANIFEST: VersionManifest = read_global_manifest();
}

pub fn versions() -> impl Iterator<Item = &'static Version> {
    MANIFEST.versions.iter()
}
/// downloads client.json for a given minecraft version and the client.json contents as a string
pub fn download_version(version: &str) -> Result<Vec<u8>, CoreError<'static>> {
    let Some(version) = versions().find(|x| x.id == version) else {
        return Err(CoreError::MinecraftVersionNotFound);
    };
    let res = utils::download::get(&version.url)?;
    Ok(res)
}
