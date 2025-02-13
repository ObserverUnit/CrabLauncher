use crate::{
    client,
    utils::{
        download::DownloadError,
        errors::{ExecutionError, InstallationError},
        MULTI_PATH_SEPRATOR,
    },
    version_manifest, ASSETS_PATH, LAUNCHER_PATH, LIBS_PATH,
};
use std::{
    borrow::Cow,
    fs::{self, File, OpenOptions},
    io::{BufReader, Seek, SeekFrom},
    path::PathBuf,
    process::{Command, Stdio},
};

use crab_launcher_api::meta::client::Client;
use serde::{Deserialize, Serialize};

use crate::{
    config::{Config, ConfigMut},
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

    /// attempts to read the config.json file for this profile
    fn read_config(&self) -> Option<Config> {
        let config_path = self.config_path();
        let config = fs::read_to_string(&config_path).ok()?;
        Some(serde_json::from_str(&config).expect("failed to deserialize config.json"))
    }

    /// returns the config used by this profile, and merges it with the global config
    pub fn get_config(&self) -> Config {
        let global_config = Config::read_global();
        if let Some(config) = self.read_config() {
            config.merge(global_config)
        } else {
            global_config
        }
    }

    /// returns a mutable reference to the config used by this profile if any
    pub fn config_mut(&mut self) -> ConfigMut {
        let config_path = self.config_path();
        self.read_config()
            .unwrap_or(Config::empty())
            .into_mut(&config_path)
    }

    pub fn read_client(&self) -> Result<Client, DownloadError> {
        let dir_path = self.dir_path();
        let client_path = dir_path.join("client.json");
        fs::create_dir_all(dir_path)?;

        let data = match fs::read_to_string(&client_path).ok() {
            Some(data) => data,
            None => {
                let version = version_manifest::download_version(&self.version)?.unwrap();
                fs::write(client_path, &version)?;
                version
            }
        };

        Ok(serde_json::from_str(&data).expect("failed to deserialize client.json"))
    }

    pub fn install(&self) -> Result<(), InstallationError> {
        let path = self.dir_path();
        let client = self.read_client()?;
        println!("downloading profile {}", self.name);
        client::install_client(client, &path)
    }

    fn classpath(&self, client: &Client) -> String {
        let libs = client.libs();

        let mut classpath = Vec::new();
        for lib in libs {
            if let Some(ref native) = lib.platform_native() {
                let path = native.sub_path.as_ref().unwrap();
                let full_path = LIBS_PATH.join(path);
                classpath.push(format!("{}", full_path.display()));
            }
            if let Some(ref artifact) = lib.downloads.artifact {
                let path = artifact.sub_path.as_ref().unwrap();
                let full_path = LIBS_PATH.join(path);
                classpath.push(format!("{}", full_path.display()));
            }
        }

        let client_jar = self.client_jar_path();
        classpath.push(format!("{}", client_jar.display()));
        classpath.join(MULTI_PATH_SEPRATOR)
    }

    /// generates the java arguments required to launch this profile
    /// NOTE: may panic if [`Self::install`] was not successfully executed first (assumes that the client.json file exists)
    fn generate_arguments(&self, config: &Config) -> Vec<String> {
        let client = self.read_client().unwrap();
        let classpath = self.classpath(&client);
        let game_dir = self.dir_path();
        let natives_dir = game_dir.join(".natives");

        let raw_args = client.arguments;
        let (mut jvm_args, mut game_args) = raw_args.into_raw();
        let regex = regex::Regex::new(r"\$\{(\w+)\}")
            .expect("failed to compile regex for parsing arguments");

        let fmt_arg = |arg: &str| {
            Some(match arg {
                "game_directory" => game_dir.to_str().unwrap(),
                "assets_root" | "game_assets" => ASSETS_PATH.to_str().unwrap(),
                "assets_index_name" => &client.assets,
                "version_name" => self.version(),
                "classpath" => classpath.as_str(),
                "natives_directory" => natives_dir.to_str().unwrap(),
                _ => config.get(arg)?,
            })
        };

        let fmt_args = |args: &mut Vec<String>| {
            for arg in args {
                let new_value = regex.replace_all(&arg, |caps: &regex::Captures| {
                    let fmt_spec = caps.get(1).unwrap().as_str();
                    fmt_arg(fmt_spec).unwrap_or_default()
                });

                if let Cow::Owned(value) = new_value {
                    *arg = value;
                }
            }
        };

        fmt_args(&mut game_args);
        fmt_args(&mut jvm_args);

        jvm_args.push(client.main_class.clone());
        [jvm_args, game_args].concat()
    }

    pub fn execute(&self) -> Result<(), ExecutionError<'static>> {
        let config = self.get_config();
        let current_java_path = config.get("current_java_path").unwrap();
        let max_ram = config.get("max_ram").unwrap();
        let min_ram = config.get("min_ram").unwrap();

        let args = self.generate_arguments(&config);

        dbg!("executing with args: {:?}", &args);
        // TODO: make use of client.arguments
        let output = Command::new(current_java_path)
            .arg(format!("-Xmx{}M", max_ram))
            .arg(format!("-Xms{}M", min_ram))
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;

        if !output.status.success() {
            return Err(ExecutionError::MinecraftError(
                output.status.code().unwrap(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Profiles {
    fd: File,
}

impl Profiles {
    pub fn path() -> PathBuf {
        LAUNCHER_PATH.join("profiles.json")
    }

    pub fn fetch_profiles(&self) -> Vec<Profile> {
        let reader = BufReader::new(&self.fd);
        serde_json::from_reader(reader).unwrap_or_default()
    }

    pub fn fetch() -> Self {
        let fd = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(Self::path())
            .unwrap();

        Self { fd }
    }

    pub fn get_named(&self, name: &str) -> Option<Profile> {
        let profiles = self.fetch_profiles();
        profiles.iter().find(|x| x.name == name).cloned()
    }

    fn write_profiles(&mut self, profiles: &[Profile]) {
        self.fd.set_len(0).unwrap();
        self.fd.seek(SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&self.fd, &profiles).unwrap();
    }
    pub fn add(&mut self, profile: Profile) {
        let mut profiles = self.fetch_profiles();
        profiles.push(profile);
        self.write_profiles(&profiles);
    }
}
