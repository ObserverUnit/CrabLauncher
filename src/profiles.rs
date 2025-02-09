use crate::{
    error::Error, utils::MULTI_PATH_SEPRATOR, ASSETS_PATH, LAUNCHER_PATH, LIBS_PATH, MANIFEST,
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    process::Command,
};

use serde::{Deserialize, Serialize};

use crate::{
    config::{Config, ConfigMut},
    json::client::Client,
    PROFILES_PATH,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Profile {
    name: String,
    version: String,
}

impl Profile {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    fn dir_path(&self) -> PathBuf {
        PROFILES_PATH.join(self.name.as_str())
    }

    fn config_path(&self) -> PathBuf {
        self.dir_path().join("config.json")
    }

    fn client_jar_path(&self) -> PathBuf {
        self.dir_path().join("client.jar")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn config(&self) -> Option<Config> {
        let config_path = self.config_path();
        let config = fs::read_to_string(&config_path).ok()?;
        Some(serde_json::from_str(&config).expect("failed to deserialize config.json"))
    }

    pub fn config_mut(&mut self) -> Option<ConfigMut> {
        let config_path = self.config_path();
        let config = self.config()?;
        Some(ConfigMut::new(config, &config_path))
    }

    pub fn client(&self) -> Client {
        let dir_path = self.dir_path();
        let client_path = dir_path.join("client.json");
        fs::create_dir_all(dir_path).unwrap();

        let data = match fs::read_to_string(&client_path).ok() {
            Some(data) => data,
            None => {
                let manifest = &*MANIFEST;
                let version = manifest.download_version(&self.version).unwrap().unwrap();
                fs::write(client_path, &version).unwrap();
                version
            }
        };

        serde_json::from_str(&data).expect("failed to deserialize client.json")
    }

    pub fn download(&self) -> Result<(), Error> {
        let path = self.dir_path();
        let client = self.client();
        println!("downloading profile {}", self.name);
        client.download(&path)
    }

    pub fn execute(&self, global_config: Config) -> Result<(), Error> {
        let path = self.dir_path();
        let client = self.client();
        let config = self.config().unwrap_or(global_config);

        let libs = client.get_req_libs();
        let mut classpath = libs
            .into_iter()
            .map(|x| LIBS_PATH.join(x))
            .map(|x| x.display().to_string())
            .collect::<Vec<_>>();

        let client_jar = self.client_jar_path();
        classpath.push(client_jar.display().to_string());

        let classpath = classpath.join(MULTI_PATH_SEPRATOR);

        let natives_path = path.join(".natives");
        let natives_path = natives_path.display();

        println!("classpath: {classpath}, java: {}", config.current_java_path);
        Command::new(&config.current_java_path)
            .arg(format!("-Xmx{}M", config.max_ram))
            .arg(format!("-Xms{}M", config.min_ram))
            .arg(format!("-Djava.library.path={natives_path}"))
            .arg("-cp")
            .arg(classpath)
            .arg(client.main_class)
            .arg("--accessToken")
            .arg(&config.access_token)
            .arg("--username")
            .arg(&config.username)
            .arg("--version")
            .arg(&self.version)
            .arg("--gameDir")
            .arg(path)
            .arg("--assetsDir")
            .arg(&*ASSETS_PATH)
            .arg("--assetIndex")
            .arg(client.assets)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        Ok(())
    }
}

#[derive(Debug)]
pub struct Profiles {
    profiles: Vec<Profile>,
    fd: File,
}

impl Profiles {
    pub fn path() -> PathBuf {
        LAUNCHER_PATH.join("profiles.json")
    }

    pub fn fetch() -> Self {
        let mut fd = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(Self::path())
            .unwrap();

        let mut buf = String::new();
        fd.read_to_string(&mut buf)
            .expect("failed to read profiles.json");

        let profiles = serde_json::from_str(buf.as_str()).unwrap_or_default();
        Self { profiles, fd }
    }

    pub fn get_named(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|x| x.name == name)
    }

    pub fn get_named_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|x| x.name == name)
    }

    pub fn add(&mut self, profile: Profile) {
        self.profiles.push(profile);
        self.update();
    }

    pub fn update(&mut self) {
        let profiles_json =
            serde_json::to_string_pretty(&self.profiles).expect("Failed to serialize profiles");

        self.fd.set_len(0).unwrap();
        self.fd.seek(SeekFrom::Start(0)).unwrap();
        self.fd
            .write_all(profiles_json.as_bytes())
            .expect("Failed to write profiles to file");
    }

    pub fn get(&self, index: usize) -> Option<&Profile> {
        self.profiles.get(index)
    }

    pub fn iter(&self) -> ProfilesIter {
        ProfilesIter {
            profiles: self,
            index: 0,
        }
    }
}

impl Drop for Profiles {
    fn drop(&mut self) {
        self.update();
    }
}

pub struct ProfilesIter<'a> {
    profiles: &'a Profiles,
    index: usize,
}

impl<'a> Iterator for ProfilesIter<'a> {
    type Item = &'a Profile;

    fn next(&mut self) -> Option<Self::Item> {
        let profile = self.profiles.get(self.index)?;
        self.index += 1;
        Some(profile)
    }
}
