use crate::json_types::{FsCacheable, UserIdLookup};
use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

const CACHE_DIRNAME: &str = ".cache";

pub fn load_user_lookup() -> UserIdLookup {
    // If it exists, load the users lookup from cache. Caching this data means
    // that we don't have to go back to the API repeatedly for user info between runs.
    let cache_directory = env::current_dir().unwrap().join(CACHE_DIRNAME);
    fs::create_dir_all(&cache_directory).unwrap();
    let mut user_id_lkup = UserIdLookup::new();
    let user_id_lkup_cache_path = cache_directory.join(user_id_lkup.cache_filename());
    if let Err(err) = user_id_lkup.uncache(&user_id_lkup_cache_path) {
        // It's not fatal. Cache file may not exist.
        println!("Unable to load user ids from cache: {:?}", err);
    } else {
        println!("Successfully loaded user ids from cache");
    }

    user_id_lkup
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
pub fn get_cacheable_file_path<T>(cacheable: &T) -> Result<CacheFileSystemPath, Box<dyn Error>>
where
    T: FsCacheable,
{
    let cache_directory = env::current_dir()?.join(CACHE_DIRNAME);
    let cache_path = cache_directory.join(cacheable.cache_filename());
    return Ok(CacheFileSystemPath {
        directory: cache_directory,
        file_path: cache_path,
    });
}

pub fn write_cache<T>(cacheable: &T) -> Result<(), Box<dyn Error>>
where
    T: FsCacheable,
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
