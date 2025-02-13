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
    pub arch: Option<Arch>,
}

impl Os {
    /// Returns true if the current platform matches the given [`Os`]
    pub fn matches(&self) -> bool {
        (self.name.is_none() || self.name == Some(crate::OS))
            && (self.arch.is_none() || self.arch == Some(crate::ARCH))
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ArgValue {
    Value(String),
    Values(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Argument {
    Arg(String),
    Rule { rules: Vec<Rule>, value: ArgValue },
}

impl Argument {
    fn into_raw(self) -> Vec<String> {
        match self {
            Argument::Arg(arg) => vec![arg],
            Argument::Rule { rules, value } => {
                if rules.iter().all(Rule::is_allowed) {
                    match value {
                        ArgValue::Value(value) => vec![value],
                        ArgValue::Values(values) => values,
                    }
                } else {
                    vec![]
                }
            }
        }
    }
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

impl Arguments {
    /// maps `Arguments` to (JVM Args, Game Args)
    /// only maps arguments that are allowed by their rules
    pub fn into_raw(self) -> (Vec<String>, Vec<String>) {
        match self {
            Arguments::Args { game, jvm } => {
                let jvm: Vec<String> = jvm.into_iter().map(Argument::into_raw).flatten().collect();
                let game = game.into_iter().map(Argument::into_raw).flatten().collect();
                (jvm, game)
            }
            Arguments::MinecraftArgs(args) => {
                let game = args.split(' ').map(|arg| arg.to_string()).collect();
                // FIXME: a little hack to have jvm args when on older versions
                // TODO: fix this when we have
                // our own meta format
                let jvm = [
                    "-Djava.library.path=${natives_directory}",
                    "-cp",
                    r"${classpath}",
                ];
                let jvm = jvm.into_iter().map(|x| x.to_string()).collect();

                (jvm, game)
            }
        }
    }
}

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct JavaVersion {
//     pub component: String,
//     pub major_version: u16,
// }

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
#[serde(rename_all = "camelCase")]
pub struct Client {
    #[serde(alias = "minecraftArguments")]
    pub arguments: Arguments,
    pub asset_index: Download,

    pub assets: String,
    pub downloads: Downloads,

    /*     pub java_version: Option<JavaVersion>, */
    pub libraries: Vec<Library>,
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
        self.libraries.iter().filter(|l| l.is_allowed())
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
