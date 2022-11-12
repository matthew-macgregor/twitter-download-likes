use reqwest::{
    header::{AUTHORIZATION, USER_AGENT},
    Response,
};
use std::env;

// https://github.com/twitterdev/Twitter-API-v2-sample-code/blob/main/Likes-Lookup/liked_tweets.py
fn create_url(user_id: &str) -> (String,) {
    // tweet_fields:
    // attachments, author_id, context_annotations,
    // conversation_id, created_at, entities, geo, id,
    // in_reply_to_user_id, lang, non_public_metrics, organic_metrics,
    // possibly_sensitive, promoted_metrics, public_metrics, referenced_tweets,
    // source, text, and withheld

    let url = format!(
        "https://api.twitter.com/2/users/{}/liked_tweets?tweet.fields=lang,author_id",
        user_id
    );
    (url,)
}

async fn prepare_request(
    token: &str,
    client: &reqwest::Client,
    url: &str,
) -> Result<Response, reqwest::Error> {
    let res = client
        .get(url)
        .header(AUTHORIZATION, token)
        .header(USER_AGENT, "v2LikedTweetsRust")
        .send()
        .await?;
    Ok(res)
}

#[tokio::main]
async fn main() {
    let token = env::var("BEARER_TOKEN").expect("BEARER_TOKEN environment variable is missing.");
    let url_tup = create_url("matsuzine");
    let client = reqwest::Client::new();
    println!("{}", url_tup.0);
    let res = prepare_request(&token, &client, &url_tup.0).await;
    println!("{:?}", res);
}
