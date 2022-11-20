use reqwest::header::{AUTHORIZATION, USER_AGENT};
use serde::de;

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

    // println!("{:?}", resp);
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
pub fn create_url_users_liked_tweets(user_id: &str, next_token: Option<String>) -> String {
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
    let pagination_token = match next_token {
        Some(next_token) => format!("&pagination_token={next_token}"),
        None => "".to_string(),
    };
    format!(
        "https://api.twitter.com/2/users/{user_id}/liked_tweets?{tweet_fields}{pagination_token}&max_results=100"
    )
}

pub fn create_url_users_by_username(usernames: &[String]) -> Result<String, TwitUrlFormatErrors> {
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
