mod cache;
mod json_types;
mod twitter;

use clap::{Parser, Subcommand};
use std::env;
use twitter as tw;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Twitter username to export
    #[arg(short, long)]
    username: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Export Twitter likes
    Export {
        /// ISO 8601 date
        #[arg(short, long)]
        not_before_date: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let token = env::var("BEARER_TOKEN").expect("BEARER_TOKEN environment variable is missing.");
    let args = Args::parse();
    let username = args.username;

    match &args.command {
        Some(Commands::Export { not_before_date }) => {
            tw::export_twitter_likes_for_username(&username, &token, not_before_date)
                .await
                .expect("Failed to export likes");

        }
        None => {}
    }


}
