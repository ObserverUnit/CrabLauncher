use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::path::Path;

use crate::java::{self};
use crate::LAUNCHER_PATH;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub min_ram: i32,
    pub max_ram: i32,

    pub width: i32,
    pub height: i32,

    pub username: String,
    pub access_token: String,

    pub current_java_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let java_list = java::list();

        Self {
            min_ram: 512,
            max_ram: 2048,
            width: 854,
            height: 480,
            username: String::from("dev"),
            access_token: String::from("0"),
            current_java_path: java_list[0].path.clone(),
        }
    }
}

impl Config {
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

    pub fn get(&self) -> &Config {
        &self.config
    }

    pub fn set_name(&mut self, name: String) {
        self.config.username = name;
    }

    pub fn set_access_token(&mut self, token: String) {
        self.config.access_token = token;
    }

    pub fn set_entry(&mut self, entry: &str, value: Option<String>) -> Result<(), &str> {
        match entry {
            "config.access_token" => {
                self.config.access_token = value.ok_or("couldnt find value")?;
            }

            "config.username" => {
                self.config.username = value.ok_or("couldnt find value")?;
            }

            _ => return Err("couldnt find entry"),
        }
        Ok(())
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
