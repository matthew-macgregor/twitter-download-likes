mod cache;
mod json_types;
mod twitter;

use clap::{Parser, Subcommand, ValueEnum};
use std::env;
use twitter as tw;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Export Twitter likes
    Export {
        /// Twitter username to export
        #[arg(short, long)]
        username: String,

        /// format %Y-%m-%d 2022-01-01
        #[arg(short, long)]
        not_before_date: Option<String>,

        #[arg(long, value_enum, default_value_t = OutputFormat::JSON)]
        format: OutputFormat,
    },

    Compile {
        /// Twitter username to export
        #[arg(short, long)]
        username: String,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    /// Outputs to JSON format
    JSON,
}

#[tokio::main]
async fn main() {
    let token = env::var("BEARER_TOKEN").expect("BEARER_TOKEN environment variable is missing.");
    let args = Args::parse();

    match &args.command {
        Some(Commands::Export {
            username,
            not_before_date,
            format,
        }) => {
            match tw::export_twitter_likes_for_username(&username, &token, not_before_date)
                .await {
                    Ok(_) => println!("Completed with success"),
                    Err(err) => println!("{:?}", err),
                }
            todo!("Do something with the output format: {:?}", format);
        }
        Some(Commands::Compile { username }) => {
            match tw::compile_twitter_exports(username) {
                Ok(_) => println!("Completed compilation successfully"),
                Err(err) => println!("{:?}", err),
            };
        }
        None => {}
    }
}
