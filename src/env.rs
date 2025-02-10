use crate::config::Config;
use crate::manifest::Manifest;
use crate::profiles::{Profile, Profiles};
use crate::utils::errors::ExecutionError;

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

    pub fn profiles(&mut self) -> &Profiles {
        &self.profiles
    }

    pub fn execute<'b>(&self, name: &'b str) -> Result<(), ExecutionError<'b>> {
        let profile = self
            .profiles
            .get_named(name)
            .ok_or(ExecutionError::ProfileDoesntExist(name))?;
        profile.download()?;
        profile.execute(self.config)?;
        Ok(())
    }

    pub fn add(&mut self, name: &str, version: &str) -> Result<(), ()> {
        let profile = Profile::new(name.to_string(), version.to_string());
        self.profiles.add(profile);
        Ok(())
    }

    pub fn edit(&mut self, name: &str, entry: &str, value: Option<String>) -> Result<(), ()> {
        let profile = self.profiles.get_named_mut(name);
        // TODO: that simply is not how it works
        if let Some(profile) = profile {
            let config = profile.config_mut();
            if let Some(mut config) = config {
                let _ = config.set_entry(entry, value);
            }
        }
        Ok(())
    }
}
