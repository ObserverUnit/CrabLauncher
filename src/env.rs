use crate::manifest::Manifest;
use crate::profiles::{Profile, Profiles};
use crate::utils::errors::ExecutionError;

#[derive(Debug)]
pub struct Env<'a> {
    profiles: Profiles,
    manifest: &'a Manifest,
}

impl<'a> Env<'a> {
    pub fn new(manifest: &'a Manifest) -> Self {
        Self {
            manifest,
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
        profile.execute()?;
        Ok(())
    }

    pub fn add(&mut self, name: &str, version: &str) -> Result<(), ()> {
        if self
            .manifest
            .versions
            .iter()
            .find(|v| &v.id == version)
            .is_none()
        {
            return Err(());
        }

        let profile = Profile::new(name.to_string(), version.to_string());
        self.profiles.add(profile);
        Ok(())
    }

    pub fn edit(&mut self, name: &str, entry: &str, value: Option<String>) -> Result<(), ()> {
        let profile = self.profiles.get_named_mut(name);
        // FIXME: that simply is not how it works
        if let Some(profile) = profile {
            let config = profile.config_mut();
            if let Some(mut config) = config {
                if let Some(value) = value {
                    config.get_entry_mut(entry).map(|x| *x = value);
                } else {
                    config.remove_entry(entry);
                }
            }
        }
        Ok(())
    }
}
