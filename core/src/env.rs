use std::path::{Path, PathBuf};

use crate::profiles::{Profile, ProfileMetadata, Profiles};
use crate::utils::errors::CoreError;
use crate::version_manifest::Manifest;

#[derive(Debug)]
pub struct Env<'a> {
    profiles: Profiles,
    manifest: Manifest,
    launcher_root: &'a Path,
    libs_root: PathBuf,
    assets_root: PathBuf,
}

impl<'a> Env<'a> {
    pub fn root(&self) -> &Path {
        self.launcher_root
    }

    pub fn libs(&self) -> &Path {
        &self.libs_root
    }

    pub fn assets(&self) -> &Path {
        &self.assets_root
    }

    pub async fn fetch_new(launcher_root: &'a Path) -> Self {
        Self {
            profiles: Profiles::fetch(launcher_root),
            manifest: Manifest::fetch(launcher_root).await,
            libs_root: launcher_root.join("libs"),
            assets_root: launcher_root.join("assets"),
            launcher_root,
        }
    }

    pub fn profiles(&mut self) -> &Profiles {
        &self.profiles
    }

    #[inline]
    fn get_profile(&self, name: &str) -> Option<Profile> {
        let profile_metadata = self.profiles.get_named(name)?;
        let profile = Profile::new(
            profile_metadata,
            self.root(),
            self.profiles.root(),
            self.libs(),
            self.assets(),
        );

        Some(profile)
    }

    pub async fn execute<'b>(&self, name: &'b str) -> Result<(), CoreError<'b>> {
        let mut profile = self
            .get_profile(name)
            .ok_or(CoreError::ProfileNotFound(name))?;

        profile.install(&self.manifest).await?;
        profile.execute()?;
        Ok(())
    }

    pub async fn add(&mut self, name: &str, version: &str) -> Result<(), CoreError<'static>> {
        let metadata = ProfileMetadata::new(name.to_owned(), version.to_owned());
        self.profiles.add(metadata);
        Ok(())
    }

    pub fn edit(&mut self, name: &str, entry: &str, value: Option<String>) -> Result<(), ()> {
        println!("setting {} entry {} to {:?}", name, entry, value);
        let mut profile = self.get_profile(name).ok_or(())?;

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
