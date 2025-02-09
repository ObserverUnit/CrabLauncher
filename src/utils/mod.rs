use serde::Deserialize;
pub const MULTI_PATH_SEPRATOR: &'static str = if cfg!(target_os = "windows") {
    ";"
} else {
    ":"
};

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

pub mod download;
pub mod errors;
pub mod zip;
