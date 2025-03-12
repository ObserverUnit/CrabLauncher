mod cli;
use clap::Parser;
use cli::Cli;
use crab_launcher_core::{env::Env, utils::errors::CoreError};
#[tokio::main]
async fn main() {
    let parse = Cli::try_parse().unwrap_or_else(|e| e.exit());
    let mut env = Env::fetch_new().await;

    match parse.command {
        cli::Commands::New(new) => env
            .add(&new.name, &new.version)
            .await
            .expect("failed to add profile"),

        cli::Commands::Edit { name, entry, value } => env
            .edit(&name, &entry, value.clone())
            .expect(&format!("failed to set {name}'s {entry} to {value:?}")),

        cli::Commands::Run { name } => match env.execute(&name).await {
            Ok(_) => println!("Minecraft exited successfully"),
            Err(err) => match err {
                CoreError::MinecraftFailure(exit_code) => {
                    eprintln!("Minecraft exited with code {}", exit_code);
                }
                CoreError::IoError(err) => {
                    eprintln!("IO error: {}", err);
                }

                CoreError::ZipError(err) => {
                    eprintln!("Failed unzipping: {}", err);
                }
                CoreError::ProfileNotFound(name) => {
                    eprintln!("Profile `{}` not found", name);
                }
                CoreError::DownloadError(err) => {
                    eprintln!("Download error: {:?}", err);
                }
                CoreError::MinecraftVersionNotFound => unreachable!(),
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
