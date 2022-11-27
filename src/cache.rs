use crate::json_types::{FsCacheable, LikedTweets, TwitLikeResponse, UserIdLookup};
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

const CACHE_DIRNAME: &str = ".cache";

pub fn load_user_lookup() -> UserIdLookup {
    // If it exists, load the users lookup from cache. Caching this data means
    // that we don't have to go back to the API repeatedly for user info between runs.
    let cache_directory = env::current_dir().unwrap().join(CACHE_DIRNAME);
    fs::create_dir_all(&cache_directory).unwrap(); // TODO
                                                   // let mut user_id_lkup = UserIdLookup::new();
    let user_id_lkup_cache_path = cache_directory.join(user_id_lkup.cache_filename());
    if let Err(err) = UserIdLookup::uncache(&user_id_lkup_cache_path) {
        // It's not fatal. Cache file may not exist.
        println!("Unable to load user ids from cache: {:?}", err);
    } else {
        println!("Successfully loaded user ids from cache");
    }

    user_id_lkup
}

pub fn load_all_liked_tweets() -> LikedTweets {
    // From the cache directory, find all cached JSON files with liked tweets.
    let cache_directory = env::current_dir().unwrap().join(CACHE_DIRNAME); // TODO
    let mut liked_tweets = LikedTweets {
        user_id: None,
        username: None,
        tweets: Vec::new(),
    };

    let paths = fs::read_dir(cache_directory).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display());
        let twit_like_resp = TwitLikeResponse::uncache(path);
        println!("Twitter Like Response from cache: {:?}", twit_like_resp);
    }

    liked_tweets
}

pub struct CacheFileSystemPath {
    pub directory: PathBuf,
    pub file_path: PathBuf,
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
pub fn get_cacheable_file_path(filename: &str) -> Result<CacheFileSystemPath, Box<dyn Error>> {
    let cache_directory = env::current_dir()?.join(CACHE_DIRNAME);
    let cache_path = cache_directory.join(filename);
    return Ok(CacheFileSystemPath {
        directory: cache_directory,
        file_path: cache_path,
    });
}

pub fn write_cache<T>(cacheable: &T) -> Result<(), Box<dyn Error>>
where
    T: FsCacheable<T>,
{
    let cacheable_file_path = get_cacheable_file_path(cacheable)?;
    let cache_directory = cacheable_file_path.directory;
    let cache_filepath = cacheable_file_path.file_path;

    fs::create_dir_all(&cache_directory)?;
    if let Err(error) = cacheable.cache(&cache_filepath) {
        println!("Failed to cache users by id: {error}");
    }

    Ok(())
}
