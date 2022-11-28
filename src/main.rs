//! Twitter Likes Exporter
/// Functions for writing/loading JSON data to disk.
mod cache;
/// Types for (de)serialization to/from JSON.
mod json_types;
/// Functions to interact with the Twitter API.
mod twitter;
/// Command line tool arguments.
mod args;

use std::env;
use args::Commands;
use chrono::NaiveDate;
use twitter as tw;

/// 
/// ```
/// export BEARER_TOKEN=REPLACE_ME
/// cargo run -- export --username matsuzine
/// cargo run -- compile --username matsuzine
/// ```
#[tokio::main]
async fn main() {
    let token = env::var("BEARER_TOKEN").expect("BEARER_TOKEN environment variable is missing.");
    let args = args::parse();

    match &args.command {
        Some(Commands::Export {
            username,
            not_before_date,
            format,
        }) => {
            // Either parse a date from the option, or get a date in prehistory.
            let not_before_date = if let Some(not_before_date) = not_before_date {
                NaiveDate::parse_from_str(&not_before_date, "%Y-%m-%d").unwrap()
            } else {
                NaiveDate::MIN
            };

            match tw::export_twitter_likes_for_username(username, &token, not_before_date)
                .await {
                    Ok(_) => println!("Completed with success"),
                    Err(err) => println!("{:?}", err),
                }
            todo!("Do something with the output format: {:?}", format);
        }
        Some(Commands::Compile { username }) => {
            match tw::compile_twitter_exports_for_username(username) {
                Ok(_) => println!("Completed compilation successfully"),
                Err(err) => println!("{:?}", err),
            };
        }
        None => {}
    }
}
