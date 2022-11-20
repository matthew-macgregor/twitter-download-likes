mod cache;
mod json_types;
mod twitter;

use crate::json_types::{TwitLikeResponse, TwitUserResponse};
use std::env;
use twitter as tw;

#[tokio::main]
async fn main() {
    let token = env::var("BEARER_TOKEN").expect("BEARER_TOKEN environment variable is missing.");
    let username: String = env::var("USERNAME").expect("USERNAME environment variable is missing.");
    let client = reqwest::Client::new();

    let url_users_by = match tw::create_url_users_by_username(&[username]) {
        Err(tw::TwitUrlFormatErrors::ExceedsLimit(msg)) => panic!("{msg}"),
        Err(tw::TwitUrlFormatErrors::NotAtMinimum(msg)) => panic!("{msg}"),
        Ok(url) => url,
    };
    let user_response = tw::send_request::<TwitUserResponse>(&token, &client, &url_users_by).await;

    let user = &user_response.data[0];
    let url_users_liked = tw::create_url_users_liked_tweets(&user.id);

    println!("username={}", &user.name);
    println!("user_id={}", &user.id,);
    println!("url_users_liked={}", url_users_liked);

    let like_response =
        tw::send_request::<TwitLikeResponse>(&token, &client, &url_users_liked).await;

    let mut user_id_lkup = cache::load_user_lookup();
    for data in like_response.data.iter() {
        // Gather all of the user_ids for the liked tweets to batch download
        if !user_id_lkup.has(&data.author_id) {
            user_id_lkup.insert(data.author_id.clone(), None);
        }
    }

    // println!("{:?}", like_response.meta);
    let meta = match &like_response.meta {
        Some(meta) => meta,
        None => panic!("Expected meta not none"),
    };

    // want to use array_chunks here but will wait for it to stabilize
    let mut missing_users: Vec<String> = Vec::new();
    for (key, value) in &user_id_lkup.users_by_id {
        println!("{} / {:?}", key, value);
        if let None = value {
            println!("Missing user: {}", key);
            missing_users.push(key.clone());
        }
    }

    // let user_ids_all = user_id_lkup.keys_to_vec();
    match tw::create_url_users_by_ids(&missing_users) {
        Err(tw::TwitUrlFormatErrors::ExceedsLimit(msg)) => panic!("{msg}"),
        Err(tw::TwitUrlFormatErrors::NotAtMinimum(msg)) => {
            println!("No users to look up: {msg}");
        }
        Ok(url) => {
            println!("{:?}", url);
            let users_response = tw::send_request::<TwitUserResponse>(&token, &client, &url).await;

            for user in users_response.data {
                user_id_lkup.insert(user.id.clone(), Some(user.clone()));
            }

            cache::write_user_lookup(&user_id_lkup);
        }
    };
}
