use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use utils::{Arch, OsName};

mod client;
mod config;
mod java;
mod version_manifest;

pub mod env;
pub mod profiles;
pub mod utils;

pub const OS: OsName = if cfg!(target_os = "windows") {
    OsName::Windows
} else if cfg!(target_os = "macos") {
    OsName::Osx
} else if cfg!(target_os = "linux") {
    OsName::Linux
} else {
    panic!("unsupported OS")
};

pub const ARCH: Arch = if cfg!(target_arch = "x86") {
    Arch::X86
} else if cfg!(target_arch = "x86_64") {
    Arch::X86_64
} else if cfg!(target_arch = "aarch64") {
    Arch::ARM64
} else {
    panic!("unsupported arch")
};

lazy_static! {
    // Paths
    pub static ref LAUNCHER_PATH: &'static Path = &Path::new("launcher");
    pub static ref LIBS_PATH: PathBuf = LAUNCHER_PATH.join("libs");
    pub static ref ASSETS_PATH: PathBuf = LAUNCHER_PATH.join("assets");
    pub static ref PROFILES_PATH: PathBuf = LAUNCHER_PATH.join("profiles");
}
