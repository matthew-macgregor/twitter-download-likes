# Twitter Favorites Exporter

- **Short Description**: Export your Twitter favorites to JSON
- **Development Status**: Beta

I primarily use Twitter to bookmark interesting projects that I might want to revisit. Since Twitter's acquisition has made its future less certain (and less palatable), I would like to get an export of my favorites for safekeeping (since they aren't included in the archive).

I've been learning Rust over the last year or so, and this project also serves as an opportunity to work on something to practice my understanding of the language in a small codebase.

These are the project goals:

- Create a command line exporter for Twitter favorites
- JSON and (optionally) Markdown output formats
- Use the project to develop my Rust skills
- Develop the project entirely in Github Codespaces to give it a try

- Starter template was adapted from: https://github.com/codespaces-examples/rust and updated to newer versions of Ubuntu and dependencies.

### Build and Run

Set the Twitter API token to `BEARER_TOKEN`.

```sh
export BEARER_TOKEN={your_twitter_token}
cargo run -- help
```

There are two steps to the process. First, export the tweets to a local cache. To export all likes by a given Twitter username:

```sh
cargo run -- export --username {your_username}
```

Limit the lookback by setting `--not-before-date 2022-01-01` (or set the date as you prefer).

Once you have exported the tweets, compile them into JSON or Markdown.

```sh
cargo run -- compile --username {your_username} --format {json,markdown}
```

### To Do

- [] Better error handling
    - Read some best practices / patterns
- [x] Pass in optional pagination token
- [x] Handle end case -- empty data[]?
- [x] Markdown output format (needs improvement)