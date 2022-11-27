use crate::{
    cache,
    json_types::{TwitLikeResponse, TwitUserResponse, UserIdLookup},
};
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use serde::de;
use std::{error::Error, path::Path};

pub async fn send_request<T>(bearer_token: &str, client: &reqwest::Client, url: &str) -> T
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

    let twit_data: T = match resp.status() {
        reqwest::StatusCode::OK => resp.json::<T>().await.unwrap(),
        _ => panic!("Bad response: {:?}", resp.text().await.unwrap()),
    };

    // println!("{:?}", twit_data);
    twit_data
}

pub enum TwitUrlFormatErrors {
    ExceedsLimit(String),
    NotAtMinimum(String),
}

// https://github.com/twitterdev/Twitter-API-v2-sample-code/blob/main/Likes-Lookup/liked_tweets.py
pub fn create_url_users_liked_tweets(user_id: &str, next_token: &Option<String>) -> String {
    // tweet_fields:
    // attachments, author_id, context_annotations,
    // conversation_id, created_at, entities, geo, id,
    // in_reply_to_user_id, lang, non_public_metrics, organic_metrics,
    // possibly_sensitive, promoted_metrics, public_metrics, referenced_tweets,
    // source, text, and withheld

    //     curl \
    // -H "Authorization: Bearer $TOKEN" \
    // "https://api.twitter.com/2/users/1446894253/liked_tweets?max_results=100&pagination_token=7140dibdnow9c7btw481sf1t9hxmmcxmseeltcdseos3c&expansions=author_id&user.fields=name&tweet.fields=attachments%2Cauthor_id%2Centities"
    let tweet_fields = "tweet.fields=created_at,lang,author_id,attachments,entities";
    let pagination_token = match next_token {
        Some(next_token) => format!("&pagination_token={next_token}"),
        None => "".to_string(),
    };
    format!(
        "https://api.twitter.com/2/users/{user_id}/liked_tweets?{tweet_fields}{pagination_token}&max_results=100"
    )
}

pub fn create_url_users_by_username(usernames: &[&str]) -> Result<String, TwitUrlFormatErrors> {
    // usernames = "usernames=TwitterDev,TwitterAPI"
    // user_fields = "user.fields=description,created_at"
    // # User fields are adjustable, options include:
    // # created_at, description, entities, id, location, name,
    // # pinned_tweet_id, profile_image_url, protected,
    // # public_metrics, url, username, verified, and withheld
    // url = "https://api.twitter.com/2/users/by?{}&{}".format(usernames, user_fields)
    if usernames.len() > 100 {
        return Err(TwitUrlFormatErrors::ExceedsLimit(
            "Number of usernames is limited to 100".to_owned(),
        ));
    } else if usernames.len() < 1 {
        return Err(TwitUrlFormatErrors::NotAtMinimum(
            "At least 1 username is required".to_owned(),
        ));
    }

    let usernames = usernames.join(",");
    Ok(format!(
        "https://api.twitter.com/2/users/by?usernames={}&user.fields=id,description,name,username,url,profile_image_url",
        usernames,
    ))
}

pub fn create_url_users_by_ids(user_ids: &[String]) -> Result<String, TwitUrlFormatErrors> {
    // created_at, description, entities, id, location, name,
    // pinned_tweet_id, profile_image_url, protected,
    // public_metrics, url, username, verified, withheld
    // curl "https://api.twitter.com/2/users?ids=3107896458,823083&user.fields=id,profile_image_url,url,username"
    if user_ids.len() > 100 {
        return Err(TwitUrlFormatErrors::ExceedsLimit(
            "Number of user_ids is limited to 100".to_owned(),
        ));
    } else if user_ids.len() < 1 {
        return Err(TwitUrlFormatErrors::NotAtMinimum(
            "At least 1 username is required".to_owned(),
        ));
    }
    let user_ids = user_ids.join(",");

    Ok(format!(
        "https://api.twitter.com/2/users?ids={user_ids}&user.fields=id,profile_image_url,url,username"
    ))
}

pub fn compile_twitter_exports(username: &str) -> Result<(), Box<dyn Error>> {
    let liked_tweets = cache::load_all_liked_tweets_from_cache(username)?;

    cache::write_cache(
        &liked_tweets,
        Path::new(&format!("liked_tweets-{username}.json"))).unwrap();

    Ok(())
}

pub async fn export_twitter_likes_for_username(
    username: &str,
    token: &str,
    not_before_date: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();

    // Look up the twitter user id by user name / handle
    let url_users_by = match create_url_users_by_username(&[username]) {
        Err(TwitUrlFormatErrors::ExceedsLimit(msg)) => panic!("{msg}"),
        Err(TwitUrlFormatErrors::NotAtMinimum(msg)) => panic!("{msg}"),
        Ok(url) => url,
    };
    let user_response = send_request::<TwitUserResponse>(&token, &client, &url_users_by).await;

    let user = &user_response.data[0];
    let mut user_id_lkup = cache::try_load_user_lookup();
    let mut next_token: Option<String> = None;
    let mut count: u64 = 0;

    loop {
        println!("Fetching the next batch of tweets...");
        let url_users_liked = create_url_users_liked_tweets(&user.id, &next_token);
        // TODO: Check here if the cache exists, skip loop if so

        let mut like_response =
            send_request::<TwitLikeResponse>(&token, &client, &url_users_liked).await;

        like_response.user = Some(user.clone());

        if let Some(tkn) = next_token {
            like_response.id = Some(tkn);
            like_response.index = Some(count);
        }

        if let Some(mut meta) = like_response.meta.clone() {
            meta.user_id = Some(user.id.clone());
            meta.username = Some(user.name.clone());
        }

        // Collect any of the users that we haven't cached previously
        for data in like_response.data.iter() {
            // Gather all of the user_ids for the liked tweets to batch download
            if !user_id_lkup.has(&data.author_id) {
                user_id_lkup.insert(data.author_id.clone(), None);
            }
        }

        let mut missing_users: Vec<String> = Vec::new();
        for (key, value) in &user_id_lkup.users_by_id {
            if let None = value {
                missing_users.push(key.clone());
            }
        }

        if missing_users.len() > 0 {
            match create_url_users_by_ids(&missing_users) {
                Err(TwitUrlFormatErrors::ExceedsLimit(msg)) => panic!("{msg}"),
                Err(TwitUrlFormatErrors::NotAtMinimum(msg)) => {
                    println!("No users to look up: {msg}");
                }
                Ok(url) => {
                    // println!("{:?}", url);
                    let users_response =
                        send_request::<TwitUserResponse>(&token, &client, &url).await;

                    for user in users_response.data {
                        user_id_lkup.insert(user.id.clone(), Some(user.clone()));
                    }

                    cache::write_cache(
                        &user_id_lkup, 
                        &UserIdLookup::fs_full_path().unwrap()
                    )?;
                }
            };
        }

        if let Some(fs_path) = like_response.fs_full_path() {
            println!("{:?}", fs_path);
            if fs_path.exists() {
                println!("Cache exists for this batch of tweets, skipping...");
            } else {
                cache::write_cache(
                    &like_response,
                    &fs_path,
                )?;
            }
        }

        if like_response.has_next_token() {
            next_token = like_response.next_token();
        } else {
            println!("No pagination token. Finished.");
            break;
        }

        if let Some(ref not_before_date) = not_before_date {
            println!("Not before: {not_before_date}");
            if like_response.has_tweets_older_than(&not_before_date) {
                println!("Reached the end date: {not_before_date}");
                break;
            } else {
                println!("Not before date not reached, continuing");
            }
        }

        count += 1;
    }

    Ok(())
}
