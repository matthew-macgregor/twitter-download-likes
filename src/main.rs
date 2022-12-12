//! Twitter Likes Exporter
/// Command line tool arguments.
mod args;
/// Functions for writing/loading JSON data to disk.
mod cache;
/// Functions to output compiled favorites.
mod dumps;
/// Functions and traits to support serialization and deserialization.
mod serialization;
mod twitter;

use args::Commands;
use chrono::NaiveDate;
use std::env;
use crate::twitter::twitter as tw;

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
            next_token,
        }) => {
            // Either parse a date from the option, or get a date in prehistory.
            let not_before_date = if let Some(not_before_date) = not_before_date {
                NaiveDate::parse_from_str(&not_before_date, "%Y-%m-%d").unwrap()
            } else {
                NaiveDate::MIN
            };

            match tw::export_twitter_likes_for_username(tw::ExportTwitterLikesParams {
                username: username.clone(),
                token: token,
                next_token: next_token.clone(),
                not_before_date: not_before_date,
            })
            .await {
                Ok(_) => println!("Completed with success"),
                Err(err) => panic!("{:?}", err),
            }
        }
        Some(Commands::Compile { username, format, }) => {
            match tw::compile_twitter_exports_for_username(username, format) {
                Ok(_) => println!("Completed compilation successfully"),
                Err(err) => println!("{:?}", err),
            };
        }
        None => {}
    }
}
