use reqwest::{
    header::{AUTHORIZATION, USER_AGENT},
    Response,
};
use serde::{de, Deserialize};
use std::collections::HashMap;
use std::env;

// https://github.com/twitterdev/Twitter-API-v2-sample-code/blob/main/Likes-Lookup/liked_tweets.py
fn create_url_users_liked_tweets(user_id: &str) -> String {
    // tweet_fields:
    // attachments, author_id, context_annotations,
    // conversation_id, created_at, entities, geo, id,
    // in_reply_to_user_id, lang, non_public_metrics, organic_metrics,
    // possibly_sensitive, promoted_metrics, public_metrics, referenced_tweets,
    // source, text, and withheld

    //     curl \
    // -H "Authorization: Bearer $TOKEN" \
    // "https://api.twitter.com/2/users/1446894253/liked_tweets?max_results=100&pagination_token=7140dibdnow9c7btw481sf1t9hxmmcxmseeltcdseos3c&expansions=author_id&user.fields=name&tweet.fields=attachments%2Cauthor_id%2Centities"
    let tweet_fields = "tweet.fields=lang,author_id,attachments,entities";
    format!(
        "https://api.twitter.com/2/users/{}/liked_tweets?{}&max_results=100",
        user_id, tweet_fields,
    )
}

fn create_url_users_by(username: &str) -> String {
    // usernames = "usernames=TwitterDev,TwitterAPI"
    // user_fields = "user.fields=description,created_at"
    // # User fields are adjustable, options include:
    // # created_at, description, entities, id, location, name,
    // # pinned_tweet_id, profile_image_url, protected,
    // # public_metrics, url, username, verified, and withheld
    // url = "https://api.twitter.com/2/users/by?{}&{}".format(usernames, user_fields)

    let user_fields = "user.fields=id,description,name,username,url,profile_image_url";
    format!(
        "https://api.twitter.com/2/users/by?usernames={}&{}",
        username, user_fields
    )
}

#[derive(Deserialize, Debug)]
struct TwitUserResponse {
    data: Vec<TwitUserDatum>,
}

#[derive(Deserialize, Debug)]
struct TwitUserDatum {
    created_at: Option<String>,
    id: String,
    name: String,
    username: String,
    url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TwitLikeMeta {
    result_count: u32,
    next_token: Option<String>,
    previous_token: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TwitLikeDatum {
    id: String,
    author_id: String,
    text: String,
    entities: Option<TwitLikeEntities>,
}

#[derive(Deserialize, Debug)]
struct TwitLikeEntities {
    urls: Option<Vec<TwitLikeUrl>>,
}

#[derive(Deserialize, Debug)]
struct TwitLikeUrl {
    url: String,
    expanded_url: String,
    display_url: String,
}

#[derive(Deserialize, Debug)]
struct TwitLikeResponse {
    data: Vec<TwitLikeDatum>,
    meta: Option<TwitLikeMeta>,
}

async fn send_twitter_request<T>(bearer_token: &str, client: &reqwest::Client, url: &str) -> T
where
    T: de::DeserializeOwned + core::fmt::Debug,
{
    let resp = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {bearer_token}"))
        .header(USER_AGENT, "MatsuzineExportLikes") // TODO: fix
        .send()
        .await
        .unwrap();

    println!("{:?}", resp);
    let twit_data: T = match resp.status() {
        reqwest::StatusCode::OK => resp.json::<T>().await.unwrap(),
        _ => panic!("Bad response: {:?}", resp.text().await.unwrap()),
    };
    println!("{:?}", twit_data);
    twit_data
}

#[tokio::main]
async fn main() {
    let token = env::var("BEARER_TOKEN").expect("BEARER_TOKEN environment variable is missing.");
    let username: String = env::var("USERNAME").expect("USERNAME environment variable is missing.");
    let client = reqwest::Client::new();

    let url_users_by = create_url_users_by(&username);
    let user_response =
        send_twitter_request::<TwitUserResponse>(&token, &client, &url_users_by).await;

    let user = &user_response.data[0];
    let url_users_liked = create_url_users_liked_tweets(&user.id);

    println!("username={}", &user.name);
    println!("user_id={}", &user.id,);
    println!("{}", url_users_liked);

    let like_response =
        send_twitter_request::<TwitLikeResponse>(&token, &client, &url_users_liked).await;
    // println!("{:?}", like_response);

    for data in like_response.data.iter() {
        println!("{:?}", data);
    }
    println!("{:?}", like_response.meta);
}
