# Twitter Favorite Exporter

- **Short Description**: Export your Twitter favorites to JSON
- **Development Status**: Alpha (WIP)

I primarily use Twitter to bookmark interesting projects that I might want to revisit. Since Twitter's acquisition has made its future less certain (and less palatable), I would like to get an export of my favorites for safekeeping (since they aren't included in the archive).

I've been learning Rust over the last year or so, and this project also serves as an opportunity to work on something to practice my understanding of the language in a small codebase.

These are the project goals:

- Create a command line exporter for Twitter favorites
- JSON and (optionally) Markdown output formats
- Use the project to develop my Rust skills
- Develop the project entirely in Github Codespaces to give it a try

- Starter template was adapted from: https://github.com/codespaces-examples/rust and updated to newer versions of Ubuntu and dependencies.

### To Do

- [] Better error handling
    - Read some best practices / patterns
- [] Pass in optional pagination token
- [] Handle end case -- empty data[]?