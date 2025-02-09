use crate::{
    utils::{self, download::DownloadError, errors::InstallationError, zip::ZipExtractor},
    ASSETS_PATH, LIBS_PATH,
};
use download::{Download, Downloads};
use rule::Rule;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::utils::{Arch, OsName};

pub mod download;
pub mod rule;

#[derive(Deserialize, Debug, Clone)]
pub struct Os {
    pub name: Option<OsName>,
    pub version: Option<String>,
    pub arch: Option<Arch>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ArgValue {
    Value(String),
    Values(Vec<String>),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Argument {
    Arg(String),
    Rule { rules: Vec<Rule>, value: ArgValue },
}
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Arguments {
    Args {
        game: Vec<Argument>,
        jvm: Vec<Argument>,
    },
    MinecraftArgs(String),
}

#[derive(Debug, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LibraryDownload {
    pub artifact: Option<Download>,
    pub classifiers: Option<HashMap<String, Download>>,
}

#[derive(Debug, Deserialize)]
pub struct Extract {
    pub exclude: Option<Vec<PathBuf>>,
}

pub type Natives = HashMap<OsName, String>;
#[derive(Debug, Deserialize)]
pub struct Library {
    pub downloads: LibraryDownload,
    pub extract: Option<Extract>,

    pub name: String,
    pub natives: Option<Natives>,
    pub rules: Option<Vec<Rule>>,
}
impl Library {
    pub fn is_allowed(&self) -> bool {
        self.rules.is_none()
            || self
                .rules
                .as_ref()
                .is_some_and(|rules| rules.iter().all(Rule::is_allowed))
    }

    // TODO: consider this when implementing our own meta format
    /// returns the native library [`Download`] required by the library for the current platform
    pub fn platform_native(&self) -> Option<&Download> {
        let natives = self.natives.as_ref()?;
        let classifiers = self.downloads.classifiers.as_ref()?;
        let mut results = natives
            .iter()
            .filter(|(os, _)| **os == crate::OS)
            .map(|(_, native)| classifiers.get(native).unwrap());
        results.next()
    }
}

#[derive(Debug, Deserialize)]
pub struct Client {
    #[serde(alias = "minecraftArguments")]
    pub arguments: Arguments,
    #[serde(rename = "assetIndex")]
    pub asset_index: Download,

    pub assets: String,
    pub downloads: Downloads,
    pub id: String,

    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,

    pub libraries: Vec<Library>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
}

// assets
#[derive(Deserialize, Debug)]
pub struct Object {
    pub hash: String,
    #[allow(unused)]
    pub size: i32,
}

#[derive(Deserialize, Debug)]
pub struct Index {
    pub objects: HashMap<String, Object>,
}

impl Client {
    fn download_assets(&self) -> Result<(), DownloadError> {
        let id = &self.assets;
        println!("Downloading assets for {}...", id);
        let indexes_dir = ASSETS_PATH.join("indexes");
        let indexes_path = indexes_dir.join(format!("{}.json", id));
        let download = self.asset_index.download_in(&indexes_path)?;

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

    pub fn libs(&self) -> impl Iterator<Item = &Library> {
        self.libraries.iter().filter(|lib| {
            lib.rules.is_none()
                || lib
                    .rules
                    .as_ref()
                    .is_some_and(|rules| rules.iter().all(Rule::is_allowed))
        })
    }

    /// installs the libraries required by current client and uses the given path as the base
    /// profile directory
    pub fn install_libs(&self, path: &Path) -> Result<(), InstallationError> {
        println!("Downloading libraries...");
        for lib in self.libs() {
            // downloading lib
            if let Some(ref artifact) = lib.downloads.artifact {
                artifact.download_in(&*LIBS_PATH)?;
            }
            // downloading natives required by lib
            if let Some(native) = lib.platform_native() {
                let bytes = native.download_in(&*LIBS_PATH)?;

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

    pub fn install(self, path: &Path) -> Result<(), InstallationError> {
        self.download_assets()?;
        self.install_libs(path)?;
        println!("Downloading client...");
        let client_path = path.join("client.jar");
        // downloading client.jar
        self.downloads.client.download_in(&client_path)?;
        println!("Done downloading client");
        Ok(())
    }
}
