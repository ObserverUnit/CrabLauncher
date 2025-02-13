use crab_launcher_api::meta::client::{Client, Download, Index};

use crate::{
    utils::{self, download::DownloadError, errors::InstallationError, zip::ZipExtractor},
    ASSETS_PATH, LIBS_PATH,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn get(download: &Download) -> Result<Vec<u8>, DownloadError> {
    utils::download::get(&download.url)
}

fn download_in(download: &Download, path: &Path) -> Result<Vec<u8>, DownloadError> {
    let full_path = if let Some(ref child) = download.sub_path {
        &path.join(child)
    } else {
        path
    };

    if full_path.exists() {
        return Ok(fs::read(full_path).unwrap());
    } else {
        let data = get(download)?;

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, &data)?;
        Ok(data)
    }
}

fn download_assets(client: &Client) -> Result<(), DownloadError> {
    let id = &client.assets;
    println!("Downloading assets for {}...", id);
    let indexes_dir = ASSETS_PATH.join("indexes");
    let indexes_path = indexes_dir.join(format!("{}.json", id));
    let download = download_in(&client.asset_index, &indexes_path)?;

    // downloading objects
    let index: Index = serde_json::from_slice(&download).unwrap();
    let objects = index.objects;
    // each object is a file with a hash as name, in a subdirectory with the first 2 letters of the hash which exists in the assets folder
    for (_, object) in objects {
        let dir_name = &object.hash[0..2];
        let dir = ASSETS_PATH.join("objects").join(dir_name);
        let path = dir.join(&object.hash);

        if path.exists() {
            continue;
        }

        fs::create_dir_all(&dir)?;
        let data = utils::download::get(&format!(
            "https://resources.download.minecraft.net/{dir_name}/{}",
            object.hash
        ))?;

        fs::write(path, data)?;
    }
    println!("Downloaded assets for {}", id);
    Ok(())
}

/// installs the libraries required by current client and uses the given path as the base
/// profile directory
fn install_libs(client: &Client, path: &Path) -> Result<(), InstallationError> {
    println!("Downloading libraries...");
    for lib in client.libs() {
        // downloading lib
        if let Some(ref artifact) = lib.downloads.artifact {
            download_in(artifact, &*LIBS_PATH)?;
        }
        // downloading natives required by lib
        if let Some(native) = lib.platform_native() {
            let bytes = download_in(native, &*LIBS_PATH)?;

            if let Some(ref extract_rules) = lib.extract {
                let natives_dir = path.join(".natives");

                let exclude = extract_rules.exclude.as_deref().unwrap_or_default();
                let paths = exclude.iter().map(PathBuf::as_path).collect::<Vec<_>>();
                let zip = ZipExtractor::new(&bytes).exclude(&paths);

                zip.extract(&natives_dir)?;
            }
        }
    }

    println!("Done downloading libraries");
    Ok(())
}

/// Installs the given client into the given path, downloading all the required assets and libraries
pub fn install_client(client: Client, path: &Path) -> Result<(), InstallationError> {
    download_assets(&client)?;
    install_libs(&client, path)?;
    println!("Downloading client...");
    let client_path = path.join("client.jar");
    // downloading client.jar
    download_in(&client.downloads.client, &client_path)?;
    println!("Done downloading client");
    Ok(())
}
