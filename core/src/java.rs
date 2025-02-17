use std::{
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::Command,
    sync::{Mutex, MutexGuard},
};

use crate::{utils::OsName, OS};
use lazy_static::lazy_static;
use regex::Regex;
use rust_search::SearchBuilder;
use semver::Version;

pub fn java_manager() -> MutexGuard<'static, JavaManager> {
    JAVA_MANAGER.lock().unwrap()
}

lazy_static! {
    static ref JAVA_MANAGER: Mutex<JavaManager> = Mutex::new(JavaManager::fetch());
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaManager {
    installations: Vec<JavaInstallation>,
}

impl JavaManager {
    fn fetch() -> Self {
        Self {
            installations: list(),
        }
    }

    pub fn latest(&self) -> Option<&JavaInstallation> {
        self.installations.first()
    }

    pub fn list(&self) -> &[JavaInstallation] {
        &self.installations
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaInstallation {
    pub path: String,
    pub version: semver::Version,
}

/// Finds all java installations on the system
fn find() -> Vec<String> {
    let search = SearchBuilder::default();
    if OS == OsName::Linux {
        let paths: Vec<PathBuf> = search
            .location("/")
            .search_input("java")
            .strict()
            .build()
            .map(PathBuf::from)
            .collect();

        paths
            .iter()
            .filter(|x| {
                x.is_file()
                    && x.metadata().is_ok_and(|m| m.permissions().mode() & 0o111 != 0) // check if an exe
                    && x.file_name().is_some_and(|n| n == "java")
            })
            .map(|x| x.to_str().unwrap().to_string())
            .collect()
    } else {
        todo!("finding java for {:?} is not yet implemented", OS)
    }
}

pub fn list() -> Vec<JavaInstallation> {
    let paths = find();
    let mut list = Vec::new();
    let regex = Regex::new(r#"version "((\d+\.\d+\.\d+)_?(\d+)?)""#).unwrap(); // ^"\d+(\.\d+)*"$
                                                                               //
    for path in paths {
        let version = Command::new(&path).arg("-version").output().unwrap();

        let version = String::from_utf8(version.stderr).unwrap();

        let captures = regex.captures(&version).unwrap();

        let version = captures[1].replace("_", "+");

        list.push(JavaInstallation {
            path,
            version: Version::parse(&version).unwrap(),
        });
    }

    sort_by_version(&mut list);
    return list;
}

fn sort_by_version(list: &mut Vec<JavaInstallation>) {
    list.sort_by(|a, b| b.version.cmp(&a.version));
}
