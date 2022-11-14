use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitUserResponse {
    pub data: Vec<TwitUserDatum>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitUserDatum {
    pub created_at: Option<String>,
    pub id: String,
    pub name: String,
    pub username: String,
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitLikeResponse {
    pub data: Vec<TwitLikeDatum>,
    pub meta: Option<TwitLikeMeta>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitLikeMeta {
    pub result_count: u32,
    pub next_token: Option<String>,
    pub previous_token: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitLikeDatum {
    pub id: String,
    pub author_id: String,
    pub text: String,
    pub entities: Option<TwitLikeEntities>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitLikeEntities {
    pub urls: Option<Vec<TwitLikeUrl>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitLikeUrl {
    pub url: String,
    pub expanded_url: String,
    pub display_url: String,
}
