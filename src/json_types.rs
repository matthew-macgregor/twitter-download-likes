use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;

use crate::cache::{get_cache_file_path, get_cache_directory_path};

/// Twitter Users v2 API returns an array of user data. TwitUserResponse
/// represents the JSON response.
#[derive(Deserialize, Serialize, Debug)]
pub struct TwitUserResponse {
    pub data: Vec<TwitUserDatum>,
}

/// Twitter Users v2 API representation of a User.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitUserDatum {
    pub created_at: Option<String>,
    pub id: String,
    pub name: String,
    pub username: String,
    pub url: Option<String>,
}

/// Twitter Likes v2 API response, with optional additional information.
#[derive(Deserialize, Serialize, Debug)]
pub struct TwitLikeResponse {
    // TODO: separate object and JSON repr
    /// Optional string used to store the next pagination token that this
    /// response represents.
    pub id: Option<String>,
    /// 0 - n page of results.
    pub index: Option<u64>,
    /// User data for the user who liked this list of tweets. This will not
    /// be present if there is no data (as opposed to an empty []).
    pub user: Option<TwitUserDatum>,
    /// Tweet data returned from the API.
    pub data: Option<Vec<TwitLikeDatum>>,
    /// Metadata for this list of tweets.
    pub meta: Option<TwitLikeMeta>,
}

impl TwitLikeResponse {
    /// `true` if the Twitter API has another page of results.
    pub fn has_next_token(&self) -> bool {
        match &self.meta {
            None => false,
            Some(meta) => matches!(&meta.next_token, Some(_)),
        }
    }

    /// The pagination token needed to retrieve the next page of results.
    pub fn next_token(&self) -> Option<String> {
        if let Some(meta) = &self.meta {
            if let Some(next_token) = &meta.next_token {
                return Some(next_token.clone());
            }
        }
        None
    }

    /// Returns `true` if this list contains any tweets which are older than
    /// the date `not_before_date`.
    pub fn has_tweets_older_than(&mut self, not_before_date: &NaiveDate) -> bool {
        let data = match &self.data {
            Some(data) => data,
            None => { 
                self.data = Some(vec![]);
                return false;
            },
        };

        if data.len() == 0 {
            return false;
        }

        // If the oldest element in the list (the last one) is older than the threshold date
        let created_at = &data.last().unwrap().created_at; // TODO: unwrap
        println!("Oldest tweet in batch: {created_at}");
        let oldest_in_list = DateTime::parse_from_rfc3339(&created_at)
            .unwrap()
            .date_naive();

        return oldest_in_list.lt(not_before_date);
    }

    // Returns an optional PathBuf to the filesystem path where this response
    // would be cached, or None if there's an error getting the current working
    // directory (which should be unusual/unexpected).
    pub fn fs_full_path(&self) -> Option<PathBuf> {
        let directory = match get_cache_directory_path() {
            Ok(d) => d,
            Err(_) => return None,
        };
        let username = match &self.user {
            Some(user) => user.username.clone(),
            None => panic!("User should never be unset in fs_full_path!"),
        };
        if let Some(id) = &self.id {
            if let Some(index) = self.index {
                return Some(
                    directory.join(format!("likes-{username}-{index}-{id}.json"))
                )
            };
        }

        let dt = Utc::now();
        let timestamp: i64 = dt.timestamp();
        Some(directory.join(format!("likes-{username}-0-{timestamp}.json")))
    }
}

impl FsCacheable<Self> for TwitLikeResponse {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        write::<Self>(path, &self)?;
        Ok(self)
    }
}

impl FsLoadable<TwitLikeResponse> for TwitLikeResponse {
    fn load(path: &Path) -> Result<TwitLikeResponse, Box<dyn Error>> {
        let resp = read::<TwitLikeResponse>(path)?;
        Ok(resp)
    }
}

/// Metadata related to the current page of Tweet results.
#[derive(Deserialize, Serialize, Debug, Clone)]
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
    pub created_at: String, // date (ISO 8601)
    pub user: Option<TwitUserDatum>,
}

impl TwitLikeDatum {
    pub fn created_at_datetime(&self) -> NaiveDate {
        let created_at = &self.created_at; // TODO: unwrap
        DateTime::parse_from_rfc3339(&created_at)
            .unwrap()
            .date_naive()
    }
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

type UsersByIdHashMap = HashMap<String, Option<TwitUserDatum>>;

#[derive(Deserialize, Serialize, Debug)]
pub struct UserIdLookup {
    pub users_by_id: UsersByIdHashMap,
}

impl Default for UserIdLookup {
    fn default() -> UserIdLookup {
        UserIdLookup {
            users_by_id: HashMap::new(),
        }
    }
}

impl UserIdLookup {
    pub fn new() -> UserIdLookup {
        UserIdLookup {
            ..Default::default()
        }
    }

    pub fn has(&self, key: &str) -> bool {
        self.users_by_id.contains_key(key)
    }

    pub fn insert(&mut self, key: String, value: Option<TwitUserDatum>) -> &Self {
        self.users_by_id.insert(key, value);
        self
    }

    pub fn fs_full_path() -> std::io::Result<PathBuf> {
        Ok(get_cache_file_path("user_id_lookup.json")?)
    }

    pub fn load_default() -> Result<UserIdLookup, Box<dyn Error>> {
        Ok(
            Self::load(&Self::fs_full_path()?)?
        )
    }
}

impl FsCacheable<UserIdLookup> for UserIdLookup {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        write::<Self>(path, &self)?;
        Ok(self)
    }
}

// TODO: Not sure if this makes sense or not
impl FsLoadable<UserIdLookup> for UserIdLookup {
    fn load(path: &Path) -> Result<UserIdLookup, Box<dyn Error>> {
        Ok(read::<UserIdLookup>(path)?)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LikedTweets {
    pub user: Option<TwitUserDatum>,
    pub tweets: Vec<TwitLikeDatum>,
}

impl Default for LikedTweets {
    fn default() -> LikedTweets {
        LikedTweets {
            user: None,
            tweets: Vec::new()
        }
    }
}

impl LikedTweets {
    pub fn new() -> LikedTweets {
        LikedTweets { ..Default::default() }
    }

    pub fn sort_by_date(&mut self) -> () {
        self.tweets.sort_by(|tw1, tw2|
            tw2.created_at_datetime().partial_cmp(&tw1.created_at_datetime()).unwrap()
        );
    }
}


impl FsCacheable<LikedTweets> for LikedTweets {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        write::<LikedTweets>(path, &self)?;
        Ok(self)
    }
}

impl FsLoadable<LikedTweets> for LikedTweets {
    fn load(path: &Path) -> Result<LikedTweets, Box<dyn Error>> {
        read::<LikedTweets>(path)
    }
}

pub trait FsCacheable<T> {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>>;
}

pub trait FsLoadable<T> {
    fn load(path: &Path) -> Result<T, Box<dyn Error>>;
}

fn write<T>(path: &Path, obj: &T) -> Result<(), Box<dyn Error>>
where
    T: Serialize,
{
    let json_str = serde_json::to_string_pretty(obj)?;
    let path_str = match path.to_str() {
        Some(s) => s,
        None => "unknown path",
    };
    fs::write(path, json_str).expect(&format!("Failed to write file {}", path_str));
    Ok(())
}

fn read<T>(path: &Path) -> Result<T, Box<dyn Error>>
where
    T: for<'a> Deserialize<'a>,
{
    let json_str = fs::read_to_string(path)?;
    let result = serde_json::from_str::<T>(&json_str)?;
    Ok(result)
}
