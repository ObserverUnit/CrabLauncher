use crate::profiles::{Profile, Profiles};
use crate::utils::errors::ExecutionError;
use crate::MANIFEST;

#[derive(Debug)]
pub struct Env {
    profiles: Profiles,
}

impl Env {
    pub fn new() -> Self {
        Self {
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
        profile.install()?;
        profile.execute()?;
        Ok(())
    }

    pub fn add(&mut self, name: &str, version: &str) -> Result<(), ()> {
        let manifest = &*MANIFEST;
        if manifest
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
        println!("setting {} entry {} to {:?}", name, entry, value);
        let mut profile = self.profiles.get_named(name).ok_or(())?;
        // FIXME: that simply is not how it works
        let mut config = profile.config_mut();
        if let Some(value) = value {
            config.set(entry, value);
        } else {
            config.remove(entry);
        }
        Ok(())
    }
}
