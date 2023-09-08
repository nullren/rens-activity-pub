use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get the actor profile from an ID
    Actor {
        #[arg(short, long)]
        id: String,
    },
}
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Actor { id }) => {
            let resp = reqwest::blocking::Client::new()
                .get(&id)
                .header(
                    "Accept",
                    "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"",
                )
                .send()
                .unwrap();
            let resp = resp.json::<rap_core::types::Actor>().unwrap();
            println!("{:#?}", resp)
        }
        None => {
            println!("Hello, world! {}", rap_core::add(2, 40));
        }
    }
}
