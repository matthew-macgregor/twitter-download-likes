use crate::twitter::json_types::{TwitLikeResponse, TwitUserResponse, UserIdLookup};
use crate::{args::OutputFormat, cache, dumps};
use chrono::NaiveDate;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use serde::de;
use std::{error::Error, path::Path};


/// Submits an HTTP (GET) request to the Twitter API.
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

    // TODO: Error Handling
    // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value:
    // reqwest::Error { kind: Decode, source: Error("missing field `data`",
    // line: 1, column: 92) }', src/twitter.rs:24:59

    let twit_data: T = match resp.status() {
        reqwest::StatusCode::OK => resp.json::<T>().await.unwrap(),
        _ => panic!("Bad response: {:?}", resp.text().await.unwrap()),
    };

    twit_data
}

pub enum TwitUrlFormatErrors {
    ExceedsLimit(String),
    NotAtMinimum(String),
}

struct TwitApiUrl {}

/// TwitApiUrl has static functions for generating the URLs to make requests to
/// the Twitter API v2.
#[allow(rustdoc::bare_urls)]
impl TwitApiUrl {
    /// Generate URL fetching liked tweets for a given user id.
    /// 
    /// # Arguments
    /// 
    /// - `user_id` id of the user who liked the tweets. Note that this is a 
    /// string repr of a 64-bit integer. [Twitter ID](https://developer.twitter.com/en/docs/twitter-ids)
    /// - `next_token` pagination token, if available.
    /// 
    /// Returns a URL.
    /// <https://developer.twitter.com/en/docs/twitter-api/users/lookup/api-reference>
    /// https://api.twitter.com/2/users/{user_id}/liked_tweets
    pub fn users_liked_tweets_url(user_id: &str, next_token: Option<&str>) -> String {
        // See example:
        // https://github.com/twitterdev/Twitter-API-v2-sample-code/blob/main/Likes-Lookup/liked_tweets.py
        // tweet_fields:
        // attachments, author_id, context_annotations,
        // conversation_id, created_at, entities, geo, id,
        // in_reply_to_user_id, lang, non_public_metrics, organic_metrics,
        // possibly_sensitive, promoted_metrics, public_metrics, referenced_tweets,
        // source, text, and withheld

        // TODO: Check if user_id is empty.
    
        let tweet_fields = "tweet.fields=created_at,lang,author_id,attachments,entities";
        let pagination_token = match next_token {
            Some(next_token) => format!("&pagination_token={next_token}"),
            None => "".to_string(),
        };
        format!(
            "https://api.twitter.com/2/users/{user_id}/liked_tweets?{tweet_fields}{pagination_token}&max_results=100"
        )
    }

    /// Generate URL to fetch user objects from Twitter usernames.
    /// 
    /// # Arguments
    /// 
    /// - `usernames` usernames for the Twitter users to look up, with a max
    /// of 100 per lookup.
    /// 
    /// # Errors
    /// 
    /// - If the number of usernames is 0 or greater than 100, returns an error.
    /// 
    /// Returns a URL.
    /// <https://developer.twitter.com/en/docs/twitter-api/users/lookup/api-reference>
    /// https://api.twitter.com/2/users/by?usernames={}
    pub fn users_by_username_url(usernames: &[&str]) -> Result<String, TwitUrlFormatErrors> {
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

    /// Generate URL to fetch user objects from Twitter user ids.
    /// 
    /// # Arguments
    /// 
    /// - `user_ids` List of users to lookup by id. Note that this is a 
    /// string repr of a 64-bit integer.
    /// 
    /// # Errors
    /// 
    /// - If the number of usernames is 0 or greater than 100, returns an error.
    /// 
    /// Returns a URL.

    /// <https://developer.twitter.com/en/docs/twitter-api/users/lookup/api-reference>
    /// https://api.twitter.com/2/users?ids={user_ids}
    pub fn users_by_ids_url(user_ids: &[String]) -> Result<String, TwitUrlFormatErrors> {
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
    
}


/// Compiles a list of liked tweets and writes them to the specified output
/// format and (optional) filename
pub fn compile_twitter_exports_for_username(
    username: &str,
    format: &OutputFormat,
    filename: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let liked_tweets = cache::load_all_liked_tweets_from_cache(username)?;
    let mut default_filename = format!("liked_tweets-{username}").to_string();
    let path = match filename {
        Some(filen) => Path::new(filen),
        None => match format {
            OutputFormat::JSON => {
                default_filename = format!("{default_filename}.json");
                Path::new(&default_filename)
            }
            OutputFormat::Markdown => {
                default_filename = format!("{default_filename}.md");
                Path::new(&default_filename)
            }
        },
    };

    // TODO: Combine match with above?
    match format {
        OutputFormat::JSON => dumps::to_json(path, &liked_tweets),
        OutputFormat::Markdown => dumps::to_markdown(path, &liked_tweets),
    }
}

/// This function exports the "liked" tweets for a given user in batches, writing
/// them to a filesystem cache.
/// 
/// # Arguments
/// 
/// - `username` Twitter username string, like "matsuzine", without @.
/// - `token` Bearer token to authenticate with the Twitter API.
/// - `next_token` Optional pagination token to get the next batch of tweets.
/// - `not_before_date` Fetch will stop when a page contains a tweet older than
/// this date.
pub async fn export_twitter_likes_for_username(
    username: &str,
    token: &str,
    next_token: Option<&str>,
    not_before_date: NaiveDate,
) -> Result<(), Box<dyn Error>> {
    // TODO: Probably makes sense to break up this function a little bit.
    // TODO: Currently this function is looping through the batches of tweets
    // and writing each batch to the filesystem. This makes it easy to just 
    // restart the process and it will pick up where the previous attempt quit.
    // On the other hand, it might be nice to give the caller more control of
    // what happens to the tweets that were loaded.
    let client = reqwest::Client::new();

    // Look up the twitter user id by user name / handle
    let url_users_by = match TwitApiUrl::users_by_username_url(&[username]) {
        Err(TwitUrlFormatErrors::ExceedsLimit(msg)) => panic!("{msg}"),
        Err(TwitUrlFormatErrors::NotAtMinimum(msg)) => panic!("{msg}"),
        Ok(url) => url,
    };
    let user_response = send_request::<TwitUserResponse>(&token, &client, &url_users_by).await;

    let user = &user_response.data[0];
    let mut user_id_lkup = cache::try_load_user_lookup();
    let mut next_token = next_token.map(|t| t.to_string());
    let mut count: u64 = 0;

    loop {
        println!("Fetching the next batch of tweets...");
        let url_users_liked =
            TwitApiUrl::users_liked_tweets_url(&user.id, next_token.as_ref().map(|t| &**t));
        // TODO: Check here if the cache exists, skip loop if so

        let mut like_response =
            send_request::<TwitLikeResponse>(&token, &client, &url_users_liked).await;

        like_response.user = Some(user.clone());

        if let Some(tkn) = next_token {
            like_response.id = Some(tkn.to_string());
            like_response.index = Some(count);
        }

        // Handle missing users
        // TODO: factor this into separate functions
        if let Some(data) = &like_response.data {
            // Collect any of the users that we haven't cached previously
            for data in data.iter() {
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
                match TwitApiUrl::users_by_ids_url(&missing_users) {
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

                        cache::write_cache(&user_id_lkup, &UserIdLookup::fs_full_path().unwrap())?;
                    }
                };
            }
        }

        if let Some(fs_path) = like_response.fs_full_path() {
            println!("{:?}", fs_path);
            if fs_path.exists() {
                println!("Cache exists for this batch of tweets, skipping...");
            } else {
                cache::write_cache(&like_response, &fs_path)?;
            }
        }

        if like_response.has_next_token() {
            next_token = like_response.next_token();
        } else {
            println!("No pagination token. Finished.");
            break;
        }

        println!("Not before: {not_before_date}");
        if like_response.has_tweets_older_than(&not_before_date) {
            println!("Reached the end date: {not_before_date}");
            break;
        } else {
            println!("Not before date not reached, continuing");
        }

        count += 1;

        // Pretty sure that the rate limit on likes requests is 75 / 15 minutes.
        println!("Waiting to avoid rate limits...");
        let twelve_seconds = std::time::Duration::from_secs(12);
        std::thread::sleep(twelve_seconds);
    }

    Ok(())
}
