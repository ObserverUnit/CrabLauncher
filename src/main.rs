use std::path::{Path, PathBuf};

use clap::Parser;

use config::Config;
use json::client::{Arch, OsName};
use lazy_static::lazy_static;
use profiles::Profile;

mod cli;
mod config;
mod env;

mod client;
mod error;
mod json;

mod java;
mod profiles;
mod utils;

use cli::Cli;
use env::Env;

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
    // Global Manifest
    static ref MANIFEST: Manifest = Manifest::get();
    static ref GLOBAL_CONFIG: Config = Config::get();
    // Paths
    pub static ref LAUNCHER_PATH: &'static Path = &Path::new("launcher");
    pub static ref LIBS_PATH: PathBuf = LAUNCHER_PATH.join("libs");
    pub static ref ASSETS_PATH: PathBuf = LAUNCHER_PATH.join("assets");
    pub static ref PROFILES_PATH: PathBuf = LAUNCHER_PATH.join("profiles");
    /// Environment
    pub static ref ENV: Env<'static> = Env::new(&*MANIFEST, &*GLOBAL_CONFIG);
}

use crate::json::manifest::Manifest;

fn main() {
    let parse = Cli::try_parse().unwrap_or_else(|e| e.exit());

    let mut env = Env::new(&*MANIFEST, &*GLOBAL_CONFIG);

    match parse.command {
        cli::Commands::New(new) => env.profiles_mut().add(Profile::new(new.name, new.version)),
        cli::Commands::Edit { name, entry, value } => {
            let profile = env.profiles_mut().get_named_mut(&name);

            if let Some(profile) = profile {
                let config = profile.config_mut();
                if let Some(mut config) = config {
                    let _ = config.set_entry(&entry, value);
                }
            }
        }

        cli::Commands::Run { name } => {
            let profiles = env.profiles_mut();
            let Some(profile) = profiles.get_named_mut(&name) else {
                eprintln!("profile {} not found", name);
                return;
            };

            println!("downloading....");
            profile.download().expect("failed to download client");
            println!("downloading: OK\nrunning....");
            profile
                .execute(GLOBAL_CONFIG.clone())
                .expect("failed to run client");
            println!("FAILED or closed...");
        }

        cli::Commands::List => {
            println!("profiles:");
            for profile in env.profiles().iter() {
                println!("{}:\tversion: {}", profile.name(), profile.version());
            }
        }
        _ => todo!(),
    }
}
