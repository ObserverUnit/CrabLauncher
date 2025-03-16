use bytes::Bytes;
use crab_launcher_api::meta::client::{Client, Download, Index};
use futures::{stream::FuturesUnordered, StreamExt};

use crate::utils::{self, download::DownloadError, errors::CoreError, zip::ZipExtractor};
use std::{
    fs,
    path::{Path, PathBuf},
};

async fn download_in(download: &Download, path: &Path) -> Result<Bytes, DownloadError> {
    let full_path = if let Some(ref child) = download.sub_path {
        &path.join(child)
    } else {
        path
    };

    if let Ok(data) = fs::read(full_path) {
        return Ok(Bytes::from(data));
    } else {
        let data = utils::download::get(&download.url).await?;

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, &data)?;
        Ok(data)
    }
}

/// TODO: benchmark
#[inline(always)]
/// Downloads the given `items`` in parallel, up to the given max `download_max`` using the given download function `download`
async fn download_futures<T, F, R, I>(to_download: I, download_max: usize, download: F) -> Vec<R>
where
    I: Iterator<Item = T>,
    F: AsyncFn(T) -> R,
{
    let (size_0, size_1) = to_download.size_hint();
    let size_hint = size_1.unwrap_or(size_0);

    let mut outputs = Vec::with_capacity(size_hint.min(10));
    let mut futures = FuturesUnordered::new();

    for item in to_download {
        futures.push(download(item));
        if futures.len() == download_max {
            let next = futures.next().await.unwrap();
            outputs.push(next);
        }
    }

    while let Some(future) = futures.next().await {
        outputs.push(future);
    }

    outputs
}

async fn download_assets(assets_root: &Path, client: &Client) -> Result<(), DownloadError> {
    let id = &client.assets;
    println!("Downloading assets for {}...", id);
    let indexes_dir = assets_root.join("indexes");
    let indexes_path = indexes_dir.join(format!("{}.json", id));
    let download = download_in(&client.asset_index, &indexes_path).await?;

    // downloading objects
    let index: Index = serde_json::from_slice(&download).unwrap();
    let objects = index.objects;

    // each object is a file with a hash as name, in a subdirectory with the first 2 letters of the hash which exists in the assets folder
    let download_object =
        async |object: crab_launcher_api::meta::client::Object| -> Result<(), DownloadError> {
            let dir_name = &object.hash[0..2];
            let dir = assets_root.join("objects").join(dir_name);
            let path = dir.join(&object.hash);

            if path.exists() {
                return Ok(());
            }

            fs::create_dir_all(&dir)?;
            let data = utils::download::get(&format!(
                "https://resources.download.minecraft.net/{dir_name}/{}",
                object.hash
            ))
            .await?;

            fs::write(path, data)?;
            Ok(())
        };

    let iter = objects.into_iter();
    let iter = iter.map(|(_, object)| object);
    let outputs = download_futures(iter, 20, download_object).await;
    for (i, output) in outputs.into_iter().enumerate() {
        if let Err(err) = output {
            println!("Failed to download object indexed {i}: {err:?}");
            return Err(err);
        }
    }

    println!("Downloaded assets for {}", id);
    Ok(())
}

/// installs the libraries required by current client and uses the given path as the base
/// profile directory
async fn install_libs(
    libs_root: &Path,
    client: &Client,
    path: &Path,
) -> Result<(), CoreError<'static>> {
    println!("Downloading libraries...");
    let download_lib =
        async |lib: &crab_launcher_api::meta::client::Library| -> Result<(), CoreError<'static>> {
            // downloading lib
            if let Some(ref artifact) = lib.downloads.artifact {
                download_in(artifact, libs_root).await?;
            }
            // downloading natives required by lib
            if let Some(native) = lib.platform_native() {
                let bytes = download_in(native, libs_root).await?;

                if let Some(ref extract_rules) = lib.extract {
                    let natives_dir = path.join(".natives");

                    let exclude = extract_rules.exclude.as_deref().unwrap_or_default();
                    let paths = exclude.iter().map(PathBuf::as_path).collect::<Vec<_>>();
                    let zip = ZipExtractor::new(&bytes).exclude(&paths);

                    zip.extract(&natives_dir)?;
                }
            }
            Ok(())
        };

    let outputs = download_futures(client.libs(), 5, download_lib).await;
    for (i, output) in outputs.into_iter().enumerate() {
        if let Err(err) = output {
            println!("Failed to download library indexed {i}: {err:?}");
            return Err(err);
        }
    }

    println!("Done downloading libraries");
    Ok(())
}

/// Installs the given client into the given path, downloading all the required assets and libraries
pub async fn install_client(
    assets_root: &Path,
    libs_root: &Path,
    client: Client,
    path: &Path,
) -> Result<(), CoreError<'static>> {
    download_assets(assets_root, &client).await?;
    install_libs(libs_root, &client, path).await?;
    println!("Downloading client...");
    let client_path = path.join("client.jar");
    // downloading client.jar
    download_in(&client.downloads.client, &client_path).await?;
    println!("Done downloading client");
    Ok(())
}
