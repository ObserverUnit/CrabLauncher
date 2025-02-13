mod cli;
use clap::Parser;
use cli::Cli;
use crab_launcher_core::{env::Env, utils::errors::ExecutionError};
fn main() {
    let parse = Cli::try_parse().unwrap_or_else(|e| e.exit());
    // TODO: move everything to a separate crate and only keep the main function
    let mut env = Env::new();

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
            for profile in env.profiles().fetch_profiles() {
                println!("{}:\tversion: {}", profile.name(), profile.version());
            }
        }
        _ => todo!(),
    }
}
