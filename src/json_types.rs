use serde::{de, Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

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

    pub fn add(&mut self, key: String, value: Option<TwitUserDatum>) -> &Self {
        self.users_by_id.insert(key, value);
        self
    }

    pub fn keys_to_vec(&self) -> Vec<String> {
        self.users_by_id.keys().cloned().collect()
    }

    pub fn cache(&self, path: &str) -> Result<&Self, Box<dyn Error>> {
        self.write(path, &self.users_by_id)?;
        Ok(self)
    }

    pub fn uncache(&mut self, path: &str) -> Result<&Self, Box<dyn Error>> {
        self.users_by_id = self.read(path)?;
        Ok(self)
    }
}

pub trait JsonCache<T>
where
    T: Serialize + de::DeserializeOwned,
{
    fn write(&self, path: &str, obj: &T) -> Result<(), Box<dyn Error>> {
        let json_str = serde_json::to_string(obj)?;
        fs::write(path, json_str).expect(format!("Failed to write file {}", path).as_str());
        Ok(())
    }

    fn read(&self, path: &str) -> Result<T, Box<dyn Error>> {
        let json_str = fs::read_to_string(path)?;
        let result = serde_json::from_str::<T>(&json_str)?;
        Ok(result)
    }
}

impl JsonCache<HashMap<String, Option<TwitUserDatum>>> for UserIdLookup {}
