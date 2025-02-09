use crate::{
    client::{download::Download, rule::Rule},
    error::Error,
    utils, LIBS_PATH,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum OsName {
    Linux,
    Windows,
    Osx,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    X86_64,
    X86,
    ARM64,
}

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
pub struct Downloads {
    pub client: Download,
    pub client_mappings: Option<Download>,
    pub server: Download,
    pub server_mappings: Option<Download>,
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

#[derive(Debug, Deserialize)]
pub struct Library {
    pub downloads: LibraryDownload,
    pub extract: Option<Extract>,

    pub name: String,
    pub natives: Option<HashMap<OsName, String>>,
    pub rules: Option<Vec<Rule>>,
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
    pub fn download(self, path: &Path) -> Result<(), Error> {
        println!("Downloading assets...");
        self.asset_index.download_indexs()?;
        println!("Downloading libraries...");
        for lib in self.libraries {
            if !(lib.rules.is_none()
                || lib
                    .rules
                    .is_some_and(|rules| rules.iter().all(Rule::is_allowed)))
            {
                continue;
            }
            // downloading lib
            if let Some(ref artifact) = lib.downloads.artifact {
                artifact.download_in(&*LIBS_PATH)?;
            }
            // downloading natives required by lib
            if let Some(ref natives) = lib.natives {
                let Some(ref classifiers) = lib.downloads.classifiers else {
                    continue;
                };

                for (os, native) in natives {
                    if *os != crate::OS {
                        continue;
                    }
                    let classifier = classifiers.get(native).unwrap();

                    let bytes = classifier.download_in(&*LIBS_PATH)?;

                    if let Some(ref extract_rules) = lib.extract {
                        let natives_dir = path.join(".natives");
                        utils::extract(&bytes, &natives_dir, extract_rules.exclude.as_deref())
                            .unwrap();
                    }

                    break;
                }
            }
        }
        println!("Downloading client...");
        let client_path = path.join("client.jar");
        // downloading client.jar
        self.downloads.client.download_in(&client_path)?;
        Ok(())
    }

    /// gets pathes of required libraries to run client
    /// all pathes are relative to `LIBS_PATH`
    pub fn get_req_libs(&self) -> Vec<PathBuf> {
        let mut libs = Vec::new();

        for lib in &self.libraries {
            if !(lib.rules.is_none()
                || lib
                    .rules
                    .as_ref()
                    .is_some_and(|rules| rules.iter().all(Rule::is_allowed)))
            {
                continue;
            }

            if let Some(ref natives) = lib.natives {
                let Some(ref classifiers) = lib.downloads.classifiers else {
                    continue;
                };

                for (os, native) in natives {
                    if os != &crate::OS {
                        continue;
                    }

                    let classifier = classifiers.get(native).unwrap();
                    let path = classifier.path.clone().unwrap();
                    libs.push(path);
                    break;
                }
            }

            if let Some(ref artifact) = lib.downloads.artifact {
                libs.push(artifact.path.clone().unwrap());
            }
        }

        return libs;
    }
}
