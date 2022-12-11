use std::{error::Error, path::Path};
use crate::{json_types::LikedTweets, cache};
use std::fs::File;
use std::io::prelude::*;

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

macro_rules! write_newlines  {
    ($file:ident) => {
        writeln!($file, "{}{}", LINE_ENDING, LINE_ENDING)
    };
}

pub fn to_json(filename: &Path, liked_tweets: &LikedTweets) -> Result<(), Box<dyn Error>> {
    // We use write_cache here, since cached data is output as JSON
    cache::write_cache(
        liked_tweets,
        filename
    )
}

pub fn to_markdown(filename: &Path, liked_tweets: &LikedTweets) -> Result<(), Box<dyn Error>> {
    let display = filename.display();

    let mut file = match File::create(&filename) {
        Err(why) => panic!("Failed to open {}: {}", display, why),
        Ok(file) => file,
    };

    for tweet in &liked_tweets.tweets {
        if let Some(user) = &tweet.user {
            let default = "".to_string();
            let url = user.url.as_ref().unwrap_or(&default);
            writeln!(file, 
                "[{}]({}) id({})<br>",
                user.name,
                url, 
                tweet.author_id
            )?;
        }

        writeln!(file, "*{}*<br>", tweet.created_at)?;
        writeln!(file, "{}<br>", tweet.text)?;
        if let Some(entities) = &tweet.entities {
            if let Some(urls) = &entities.urls {
                for url in urls {
                    writeln!(file, " - [{}]({})", url.display_url, url.expanded_url)?;
                }
            }
        }

        write_newlines!(file)?;
    }

    Ok(())
}