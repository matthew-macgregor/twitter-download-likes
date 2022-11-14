mod json_types;
mod twitter;

use crate::json_types::{TwitLikeResponse, TwitUserDatum, TwitUserResponse};
use std::collections::HashMap;
use std::env;
use twitter as tw;

#[tokio::main]
async fn main() {
    let token = env::var("BEARER_TOKEN").expect("BEARER_TOKEN environment variable is missing.");
    let username: String = env::var("USERNAME").expect("USERNAME environment variable is missing.");
    let client = reqwest::Client::new();

    let url_users_by = match tw::create_url_users_by_username(&[username]) {
        Err(tw::TwitUrlFormatErrors::ExceedsLimit(msg)) => panic!("{msg}"),
        Ok(url) => url,
    };
    let user_response = tw::send_request::<TwitUserResponse>(&token, &client, &url_users_by).await;

    let user = &user_response.data[0];
    let url_users_liked = tw::create_url_users_liked_tweets(&user.id);

    println!("username={}", &user.name);
    println!("user_id={}", &user.id,);
    println!("{}", url_users_liked);

    let like_response =
        tw::send_request::<TwitLikeResponse>(&token, &client, &url_users_liked).await;
    // println!("{:?}", like_response);

    let mut user_id_map: HashMap<String, Option<TwitUserDatum>> = HashMap::new();
    for data in like_response.data.iter() {
        println!("{:?}", data);
        // Gather all of the user_ids for the liked tweets to batch download
        if !user_id_map.contains_key(&data.author_id) {
            user_id_map.insert(data.author_id.clone(), None);
        }
    }

    println!("{:?}", like_response.meta);
    let meta = match &like_response.meta {
        Some(meta) => meta,
        None => panic!("Expected meta not none"),
    };

    // want to use array_chunks here but will wait for it to stabilize
    let user_ids_all: Vec<String> = user_id_map.keys().cloned().collect();
    // let user_ids_batch = &user_ids_all[0..100];

    let url_users_by_ids = match tw::create_url_users_by_ids(&user_ids_all[0..user_ids_all.len()]) {
        Err(tw::TwitUrlFormatErrors::ExceedsLimit(msg)) => panic!("{msg}"),
        Ok(url) => url,
    };

    println!("{:?}", url_users_by_ids);
    let users_response =
        tw::send_request::<TwitUserResponse>(&token, &client, &url_users_by_ids).await;
    println!("{:?}", users_response);
}
