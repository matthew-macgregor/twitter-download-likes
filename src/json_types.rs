use chrono::{DateTime, NaiveDate};
use serde::{de, Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cache::get_cacheable_file_path;

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitUserResponse {
    pub data: Vec<TwitUserDatum>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitUserDatum {
    pub created_at: Option<String>,
    pub id: String,
    pub name: String,
    pub username: String,
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitLikeResponse {
    // TODO: separate object and JSON repr
    pub id: Option<String>,
    pub index: Option<u64>,
    pub data: Vec<TwitLikeDatum>,
    pub meta: Option<TwitLikeMeta>,
}

impl TwitLikeResponse {
    pub fn has_next_token(&self) -> bool {
        match &self.meta {
            None => false,
            Some(meta) => matches!(&meta.next_token, Some(_)),
        }
    }

    pub fn next_token(&self) -> Option<String> {
        if let Some(meta) = &self.meta {
            if let Some(next_token) = &meta.next_token {
                return Some(next_token.clone());
            }
        }
        None
    }

    pub fn has_tweets_older_than(&self, not_before_date: &str) -> bool {
        if self.data.len() == 0 {
            return false;
        }
        // If the oldest element in the list (the last one) is older than the threshold date
        let created_at = &self.data.last().unwrap().created_at; // TODO: unwrap
        println!("Oldest tweet in batch: {created_at}");
        let oldest_in_list = DateTime::parse_from_rfc3339(&created_at)
            .unwrap()
            .date_naive();
        let threshold_date = NaiveDate::parse_from_str(not_before_date, "%Y-%m-%d").unwrap();
        return oldest_in_list.lt(&threshold_date);
    }
}

impl FsCacheable<&Self> for TwitLikeResponse {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        write::<&Self>(path, &self)?;
        Ok(self)
    }
}

impl FsLoadable<TwitLikeResponse> for TwitLikeResponse {
    fn load(path: &Path) -> Result<TwitLikeResponse, Box<dyn Error>> {
        let resp = read::<TwitLikeResponse>(path)?;
        Ok(resp)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitLikeMeta {
    pub result_count: u32,
    pub next_token: Option<String>,
    pub previous_token: Option<String>,
    pub user_id: Option<String>,
    pub username: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TwitLikeDatum {
    pub id: String,
    pub author_id: String,
    pub text: String,
    pub entities: Option<TwitLikeEntities>,
    pub created_at: String, // date (ISO 8601)
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
    pub filename: String,
}

impl Default for UserIdLookup {
    fn default() -> UserIdLookup {
        UserIdLookup {
            users_by_id: HashMap::new(),
            filename: "user_id_lookup.json".to_string(),
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

    pub fn fs_full_path(&self) -> Option<PathBuf> {
        match get_cacheable_file_path(&self.filename) {
            Ok(fs_path) => Some(fs_path.file_path),
            Err(_) => None,
        }
    }
}

impl FsCacheable<UsersByIdHashMap> for UserIdLookup {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        write::<UsersByIdHashMap>(path, &self.users_by_id)?;
        Ok(self)
    }
}

impl FsLoadable<UserIdLookup> for UserIdLookup {
    fn load(path: &Path) -> Result<UserIdLookup, Box<dyn Error>> {
        let mut user_id_lookup = UserIdLookup {
            ..Default::default()
        };
        user_id_lookup.users_by_id = read::<UsersByIdHashMap>(path)?;
        Ok(user_id_lookup)
    }

    // fn cache_filename(&self) -> String {
    //     "user_id_lookup.json".to_string()
    // }

    // fn cache_fullpath(&self) -> Option<PathBuf>
    // where
    //     Self: Sized,
    // {
    //     match get_cacheable_file_path(self) {
    //         Ok(fs_path) => Some(fs_path.file_path),
    //         Err(_) => None,
    //     }
    // }

    // fn cache_exists(&self) -> bool
    // where
    //     Self: Sized,
    // {
    //     let Some(fullpath) = self.cache_fullpath() else { return false };
    //     Path::exists(&fullpath)
    // }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LikedTweets {
    pub username: Option<String>,
    pub user_id: Option<String>,
    pub tweets: Vec<TwitLikeDatum>,
}

impl FsCacheable<LikedTweets> for LikedTweets {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        write::<Self>(path, &self)?;
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
    // fn cache_filename(&self) -> String;
    // fn cache_fullpath(&self) -> Option<PathBuf>
    // where
    //     Self: Sized,
    // {
    //     match get_cacheable_file_path(self) {
    //         Ok(fs_path) => Some(fs_path.file_path),
    //         Err(_) => None,
    //     }
    // }
    // fn cache_exists(&self) -> bool
    // where
    //     Self: Sized,
    // {
    //     let Some(fullpath) = self.cache_fullpath() else { return false };
    //     Path::exists(&fullpath)
    // }
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
