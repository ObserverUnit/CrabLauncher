use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum OsName {
    Linux,
    Windows,
    Osx,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    X86_64,
    X86,
    ARM64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Os {
    pub name: Option<OsName>,
    pub arch: Option<Arch>,
}

impl Os {
    /// Returns true if the current platform matches the given [`Os`]
    pub fn matches(&self) -> bool {
        (self.name.is_none() || self.name == Some(crate::OS))
            && (self.arch.is_none() || self.arch == Some(crate::ARCH))
    }
}
