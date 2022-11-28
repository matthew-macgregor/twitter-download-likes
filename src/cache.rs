use crate::json_types::{FsCacheable, LikedTweets, TwitLikeResponse, UserIdLookup, FsLoadable};
use std::{error::Error, fmt};
use std::path::{PathBuf, Path};
use std::{env, fs, io};

const CACHE_DIRNAME: &str = ".cache";

#[derive(Debug)]
pub enum CacheLoadError {
    InvalidDefaultPath(String),
    File,
    NoTweets(String),
}

impl Error for CacheLoadError {}
impl fmt::Display for CacheLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn load_user_lookup() -> Result<UserIdLookup, Box<dyn Error>> {
    // If it exists, load the users lookup from cache. Caching this data means
    // that we don't have to go back to the API repeatedly for user info between runs.
    let cache_directory = get_cache_directory_path()?;
    fs::create_dir_all(&cache_directory)?;
    let full_path = UserIdLookup::fs_full_path()?;
    let user_id_lkup = UserIdLookup::load(&full_path)?;
    Ok(user_id_lkup)
}

pub fn try_load_user_lookup() -> UserIdLookup {
    match load_user_lookup() {
        Ok(uil) => uil,
        Err(err) => {
            println!("try_load_user_lookup: error {:?}", err);
            UserIdLookup::new()
        }
    }
}

/// Loads tweets previously cached in the .cache directory into a collection/
/// 
/// # Arguments
///
/// * `username` - A string slice representing the twitter username (not user id).
pub fn load_all_liked_tweets_from_cache(username: &str) -> Result<LikedTweets, Box<dyn Error>> {
    // From the cache directory, find all cached JSON files with liked tweets.
    let cache_directory = env::current_dir()?.join(CACHE_DIRNAME); // TODO
    let paths = fs::read_dir(cache_directory)?;
    let mut liked_tweets = LikedTweets::new();
    let user_id_lkup = UserIdLookup::load_default()?;

    for path in paths {
        let path = path.unwrap().path();
        if let Some(filen) = path.file_name() {
            if filen.to_str().unwrap().starts_with(
                &format!("likes-{username}-")
            ) {
                println!("Loaded: {}", path.display());
                let twit_like_resp = TwitLikeResponse::load(&path)?;
                if let None = liked_tweets.user {
                    liked_tweets.user = twit_like_resp.user;
                }
                for mut datum in twit_like_resp.data {
                    let user = match user_id_lkup.users_by_id.get(&datum.author_id) {
                        Some(user_opt) => user_opt.as_ref().unwrap(),
                        None => panic!("Expected user data for {}", &datum.author_id),
                    };
                    datum.user = Some(user.clone());
                    liked_tweets.tweets.push(datum);
                }
            }
        }
    }

    if liked_tweets.tweets.is_empty() {
        return Err(Box::new(CacheLoadError::NoTweets(
            format!(
                "No tweets were found for user '{}'. Did you mean to run `export` first?",
                username
            ),
        )));
    }

    liked_tweets.sort_by_date();
    Ok(liked_tweets)
}

/// Gets the filesystem path for this cacheable type.
/// Return the cache directory path, followed by the cache file path.
///
/// # Panics
///
/// Panics if unable to get the current working directory.
///
/// # Errors
///
/// This function will return an error if no cache filesystem path is available.
pub fn get_cache_file_path(filename: &str) -> std::io::Result<PathBuf> {
    Ok(get_cache_directory_path()?.join(filename))
}

pub fn get_cache_directory_path() -> io::Result<PathBuf> {
    Ok(env::current_dir()?.join(CACHE_DIRNAME))
}

pub fn write_cache<T>(cacheable: &T, file_path: &Path) -> Result<(), Box<dyn Error>>
where
    T: FsCacheable<T>,
{
    fs::create_dir_all(file_path.parent().unwrap())?;
    cacheable.cache(file_path)?;
    Ok(())
}
