use crate::config::Config;
use crate::json::manifest::Manifest;
use crate::profiles::Profiles;

#[derive(Debug)]
pub struct Env<'a> {
    profiles: Profiles,
    manifest: &'a Manifest,
    config: &'a Config,
}

impl<'a> Env<'a> {
    pub fn new(manifest: &'a Manifest, config: &'a Config) -> Self {
        Self {
            manifest,
            config,
            profiles: Profiles::fetch(),
        }
    }

    pub fn get_url(&self, ver: &String) -> &String {
        &self
            .manifest
            .versions
            .iter()
            .find(|v| &v.id == ver)
            .unwrap()
            .url
    }

    pub fn profiles(&mut self) -> &Profiles {
        &self.profiles
    }

    pub fn profiles_mut(&mut self) -> &mut Profiles {
        &mut self.profiles
    }
    //
    // // TODO: ADD ERRORS
    // pub fn edit_profile(&mut self, name: &str, entry: &str, value: Option<String>) {
    //     let profile = self.profile
    //
    //     if profile.is_none() {
    //         panic!("couldnt find profile with name {name}");
    //     }
    //
    //     let profile = profile.unwrap();
    //
    //     let config = if profile.config.is_some() {
    //         profile.config.as_mut().unwrap()
    //     } else {
    //         &mut self.config
    //     };
    //
    //     let value = value.unwrap_or_default();
    //
    //     match entry.as_str() {
    //         "config.access_token" => {
    //             config.access_token = value;
    //         }
    //
    //         "config.username" => {
    //             config.username = value;
    //         }
    //
    //         "config.current_java_path" => {
    //             config.current_java_path = value;
    //         }
    //
    //         "config.max_ram" => {
    //             config.max_ram = value.parse().unwrap();
    //         }
    //
    //         "config.min_ram" => {
    //             config.min_ram = value.parse().unwrap();
    //         }
    //
    //         "java" => {
    //             println!("available java installations:");
    //
    //             for (index, java) in config.java_list.iter().enumerate() {
    //                 println!(
    //                     "{index}:\n\tpath: {}\n\tversion: {}",
    //                     java.path, java.version
    //                 );
    //             }
    //
    //             let mut buffer = String::new();
    //
    //             println!("enter the index of the java installation you want to use: ");
    //             stdin().read_line(&mut buffer).unwrap();
    //
    //             let index: usize = buffer.trim().parse().unwrap();
    //
    //             let java = config.java_list.get(index);
    //
    //             if java.is_none() {
    //                 panic!("couldnt find java installation with index {index}");
    //             }
    //
    //             let java = java.unwrap().clone();
    //
    //             config.current_java_path = java.path.clone();
    //         }
    //
    //         _ => panic!("couldnt find entry {entry} in profile {name}"),
    //     }
    //
    //     let path = format!("{PROFILES_DIR}{name}/config.json");
    //
    //     fs::write(path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
    // }
}
