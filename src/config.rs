use serde::{Deserialize, Serialize};
use velcro::hash_map_from;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::java::{self};
use crate::LAUNCHER_PATH;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config(HashMap<String, String>);

impl Default for Config {
    fn default() -> Self {
        let java_list = java::list();
        Self(hash_map_from! {
            "min_ram": "512",
            "max_ram": "2048",
            "auth_player_name": "dev",
            "auth_access_token": "0",
            "current_java_path": &java_list[0].path,
        })
    }
}

impl Config {
    /// reads the global config
    pub fn get() -> Self {
        let path = LAUNCHER_PATH.join("config.json");

        let config = if !path.exists() {
            let config = Self::default();
            let file = File::create(path).unwrap();

            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &config).unwrap();
            config
        } else {
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        };

        config
    }

    pub fn get_entry(&self, entry: &str) -> Option<&str> {
        self.0.get(entry).map(|x| x.as_str())
    }

    pub fn get_entry_mut(&mut self, entry: &str) -> Option<&mut String> {
        self.0.get_mut(entry)
    }

    pub fn remove_entry(&mut self, entry: &str) {
        self.0.remove(entry);
    }

    pub fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
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
        let fd = File::options().write(true).open(path).unwrap();
        Self {
            config,
            fd,
            marker: PhantomData,
        }
    }

    pub fn save(&mut self) {
        self.fd.set_len(0).unwrap();
        self.fd.seek(SeekFrom::Start(0)).unwrap();
        self.fd
            .write_all(
                serde_json::to_string_pretty(&self.config)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
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
