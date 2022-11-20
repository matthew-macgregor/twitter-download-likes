use crate::json_types::UserIdLookup;
use std::{env, fs};

const CACHE_DIRNAME: &str = ".cache";

pub fn load_user_lookup() -> UserIdLookup {
    // If it exists, load the users lookup from cache. Caching this data means
    // that we don't have to go back to the API repeatedly for user info between runs.
    let cache_directory = env::current_dir().unwrap().join(CACHE_DIRNAME);
    fs::create_dir_all(&cache_directory).unwrap();
    let mut user_id_lkup = UserIdLookup::new();
    let user_id_lkup_cache_path = cache_directory.join(&user_id_lkup.cache_filename);
    if let Err(err) = user_id_lkup.uncache(&user_id_lkup_cache_path) {
        // It's not fatal. Cache file may not exist.
        println!("Unable to load user ids from cache: {:?}", err);
    } else {
        println!("Successfully loaded user ids from cache");
    }

    user_id_lkup
}

pub fn write_user_lookup(user_id_lkup: &UserIdLookup) {
    let cache_directory = env::current_dir().unwrap().join(CACHE_DIRNAME);
    fs::create_dir_all(&cache_directory).unwrap();
    let user_id_lkup_cache_path = cache_directory.join(&user_id_lkup.cache_filename);

    match user_id_lkup.cache(&user_id_lkup_cache_path) {
        Err(error) => println!("Failed to cache users by id: {error}"),
        _ => (),
    };
}
