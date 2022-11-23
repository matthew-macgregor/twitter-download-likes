mod cache;
mod json_types;
mod twitter;

use std::env;
use twitter as tw;

#[tokio::main]
async fn main() {
    let token = env::var("BEARER_TOKEN").expect("BEARER_TOKEN environment variable is missing.");
    let username: String = env::var("NAME").expect("NAME environment variable is missing.");

    tw::export_twitter_likes_for_username(&username, &token)
        .await
        .expect("Failed to export likes");
}
