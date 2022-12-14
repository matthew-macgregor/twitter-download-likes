use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Export Twitter likes
    Export {
        /// Twitter username to export
        #[arg(short, long)]
        username: String,

        /// format %Y-%m-%d 2022-01-01
        #[arg(short, long)]
        not_before_date: Option<String>,

        /// Pagination token for next page of results
        #[arg(long)]
        next_token: Option<String>,
    },

    Compile {
        /// Twitter username to export
        #[arg(short, long)]
        username: String,

        /// Output format (defaults to JSON)
        #[arg(long, value_enum, default_value_t = OutputFormat::JSON)]
        format: OutputFormat,

        #[arg(long)]
        filename: Option<String>,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    /// Outputs to JSON format
    JSON,
    /// Outputs to Markdown format
    Markdown,
}

pub fn parse() -> Args {
    Args::parse()
}