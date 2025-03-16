use serde::{Deserialize, Serialize};
use velcro::hash_map_from;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use crate::java;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config(HashMap<String, String>);

impl Config {
    fn create_default() -> Result<Self, std::io::Error> {
        let java_list = java::java_manager()
            .latest()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No Java found",
            ))?
            .clone();

        Ok(Self(hash_map_from! {
            "min_ram": "512",
            "max_ram": "2048",
            "auth_player_name": "dev",
            "auth_access_token": "0",
            "current_java_path": java_list.path,
        }))
    }
}

impl Config {
    pub fn new(map: HashMap<String, String>) -> Self {
        Self(map)
    }

    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    fn global_config_path(launcher_root: &Path) -> PathBuf {
        launcher_root.join("config.json")
    }

    /// Reads the global config and returns a memory read-only copy of it
    pub fn read_global(launcher_root: &Path) -> Result<Self, std::io::Error> {
        let path = Self::global_config_path(launcher_root);

        let config = if !path.exists() {
            let config = Self::create_default()?;
            let file = File::create(path)?;
            serde_json::to_writer_pretty(file, &config).unwrap();
            config
        } else {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        };

        Ok(config)
    }

    pub fn get(&self, entry: &str) -> Option<&str> {
        self.0.get(entry).map(|x| x.as_str())
    }

    /// Returns a new read-only config with the entries of `self` and `other` merged, favoring `self` over `other`
    pub fn merge(self, mut other: Self) -> Self {
        other.0.extend(self.0);
        other
    }

    pub fn into_mut<'a, 'b>(self, path: &'b Path) -> ConfigMut<'a> {
        ConfigMut::new(self, path)
    }
}

#[derive(Debug)]
pub struct ConfigMut<'a> {
    config: Config,
    fd: File,
    marker: PhantomData<&'a mut Config>,
}

impl<'a> ConfigMut<'a> {
    pub fn new(config: Config, path: &Path) -> Self {
        let fd = File::options().write(true).create(true).open(path).unwrap();
        Self {
            config,
            fd,
            marker: PhantomData,
        }
    }

    pub fn set(&mut self, entry: &str, value: String) {
        self.0.insert(entry.to_string(), value);
    }

    pub fn remove(&mut self, entry: &str) {
        self.0.remove(entry);
    }

    pub fn save(&mut self) {
        self.fd.set_len(0).unwrap();
        self.fd.seek(SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&self.fd, &self.config).unwrap();
    }
}
impl<'a> Drop for ConfigMut<'a> {
    fn drop(&mut self) {
        self.save();
    }
}

impl<'a> Deref for ConfigMut<'a> {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl<'a> DerefMut for ConfigMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}
