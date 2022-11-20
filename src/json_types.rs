use serde::{de, Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub data: Vec<TwitLikeDatum>,
    pub meta: Option<TwitLikeMeta>,
    cache_filename: Option<String>,
}

impl TwitLikeResponse {
    pub fn generate_cache_filename(&mut self) {
        if let Some(meta) = &self.meta {
            if let Some(next_token) = &meta.next_token {
                self.cache_filename = Some(format!("{}.json", next_token.clone()));
            }
        }
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

    fn cache_filename(&self) -> Option<&String> {
        self.cache_filename.as_ref()
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
    cache_filename: Option<String>,
}

impl UserIdLookup {
    pub fn new() -> UserIdLookup {
        UserIdLookup {
            users_by_id: HashMap::new(),
            cache_filename: Some("user_id_lookup.json".to_string()),
        }
    }

    pub fn has(&self, key: &str) -> bool {
        self.users_by_id.contains_key(key)
    }

    pub fn insert(&mut self, key: String, value: Option<TwitUserDatum>) -> &Self {
        self.users_by_id.insert(key, value);
        self
    }

    pub fn keys_to_vec(&self) -> Vec<String> {
        self.users_by_id.keys().cloned().collect()
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

    fn cache_filename(&self) -> Option<&String> {
        self.cache_filename.as_ref()
    }
}

pub trait FsCacheable {
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>>;
    fn uncache(&mut self, path: &Path) -> Result<&Self, Box<dyn Error>>;
    fn cache_filename(&self) -> Option<&String>;
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
