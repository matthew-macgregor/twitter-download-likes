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
}

impl JsonCache<TwitLikeResponse> for TwitLikeResponse {}
impl FsCacheable for TwitLikeResponse {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        self.write(path, &self)?;
        Ok(self)
    }

    fn uncache(&mut self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        *self = self.read(path)?;
        Ok(self)
    }

    fn cache_filename(&self) -> String {
        let idx = self.index.unwrap_or_default();
        match &self.id {
            Some(tkn) => format!("liked-tweets-{}-{tkn}.json", idx),
            None => format!("liked-tweets-{}-head.json", idx).to_string(),
        }
    }
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

#[derive(Deserialize, Serialize, Debug)]
pub struct UserIdLookup {
    pub users_by_id: HashMap<String, Option<TwitUserDatum>>,
}

impl UserIdLookup {
    pub fn new() -> UserIdLookup {
        UserIdLookup {
            users_by_id: HashMap::new(),
        }
    }

    pub fn has(&self, key: &str) -> bool {
        self.users_by_id.contains_key(key)
    }

    pub fn insert(&mut self, key: String, value: Option<TwitUserDatum>) -> &Self {
        self.users_by_id.insert(key, value);
        self
    }
}

impl FsCacheable for UserIdLookup {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        self.write(path, &self.users_by_id)?;
        Ok(self)
    }

    fn uncache(&mut self, path: &Path) -> Result<&Self, Box<dyn Error>> {
        self.users_by_id = self.read(path)?;
        Ok(self)
    }

    fn cache_filename(&self) -> String {
        "user_id_lookup.json".to_string()
    }
}

pub trait FsCacheable {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>>;
    fn uncache(&mut self, path: &Path) -> Result<&Self, Box<dyn Error>>;
    fn cache_filename(&self) -> String;
    fn cache_fullpath(&self) -> Option<PathBuf>
    where
        Self: Sized,
    {
        match get_cacheable_file_path(self) {
            Ok(fs_path) => Some(fs_path.file_path),
            Err(_) => None,
        }
    }
    fn cache_exists(&self) -> bool
    where
        Self: Sized,
    {
        let Some(fullpath) = self.cache_fullpath() else { return false };
        Path::exists(&fullpath)
    }
}

pub trait JsonCache<T>
where
    T: Serialize + de::DeserializeOwned,
{
    fn write(&self, path: &Path, obj: &T) -> Result<(), Box<dyn Error>> {
        let json_str = serde_json::to_string_pretty(obj)?;
        let path_str = match path.to_str() {
            Some(s) => s,
            None => "unknown path",
        };
        fs::write(path, json_str).expect(&format!("Failed to write file {}", path_str));
        Ok(())
    }

    fn read(&self, path: &Path) -> Result<T, Box<dyn Error>> {
        let json_str = fs::read_to_string(path)?;
        let result = serde_json::from_str::<T>(&json_str)?;
        Ok(result)
    }
}

impl JsonCache<HashMap<String, Option<TwitUserDatum>>> for UserIdLookup {}
