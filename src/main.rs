use std::path::{Path, PathBuf};

use clap::Parser;

use config::Config;
use lazy_static::lazy_static;
use utils::{errors::ExecutionError, Arch, OsName};

mod cli;
mod config;
mod env;

mod client;
mod manifest;

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
}

use manifest::Manifest;

fn main() {
    let parse = Cli::try_parse().unwrap_or_else(|e| e.exit());
    // TODO: move everything to a separate crate and only keep the main function
    let mut env = Env::new(&*MANIFEST);

    match parse.command {
        cli::Commands::New(new) => env
            .add(&new.name, &new.version)
            .expect("failed to add profile"),

        cli::Commands::Edit { name, entry, value } => env
            .edit(&name, &entry, value.clone())
            .expect(&format!("failed to set {name}'s {entry} to {value:?}")),

        cli::Commands::Run { name } => match env.execute(&name) {
            Ok(_) => println!("Minecraft exited successfully"),
            Err(err) => match err {
                ExecutionError::MinecraftError(exit_code) => {
                    eprintln!("Minecraft exited with code {}", exit_code);
                }
                ExecutionError::IoError(err) => {
                    eprintln!("IO error: {}", err);
                }
                ExecutionError::ProfileDoesntExist(name) => {
                    eprintln!("profile {} not found", name);
                }
                ExecutionError::InstallationError(err) => {
                    eprintln!("installation error: {:?}", err);
                }
            },
        },
        cli::Commands::List => {
            println!("profiles:");
            for profile in env.profiles().iter() {
                println!("{}:\tversion: {}", profile.name(), profile.version());
            }
        }
        _ => todo!(),
    }
}
