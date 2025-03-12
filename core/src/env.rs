use crate::profiles::{Profile, Profiles};
use crate::utils::errors::CoreError;
use crate::version_manifest::Manifest;

#[derive(Debug)]
pub struct Env {
    profiles: Profiles,
    manifest: Manifest,
}

impl Env {
    pub async fn fetch_new() -> Self {
        Self {
            profiles: Profiles::fetch(),
            manifest: Manifest::fetch().await,
        }
    }

    pub fn profiles(&mut self) -> &Profiles {
        &self.profiles
    }

    pub async fn execute<'b>(&self, name: &'b str) -> Result<(), CoreError<'b>> {
        let profile = self
            .profiles
            .get_named(name)
            .ok_or(CoreError::ProfileNotFound(name))?;
        profile.install(&self.manifest).await?;
        profile.execute()?;
        Ok(())
    }

    pub async fn add(&mut self, name: &str, version: &str) -> Result<(), CoreError<'static>> {
        let profile =
            Profile::create(&self.manifest, name.to_string(), version.to_string()).await?;
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
