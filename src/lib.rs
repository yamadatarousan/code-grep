pub mod cli;
pub mod config;
pub mod matcher;
pub mod output;
pub mod parser;
pub mod replacer;
pub mod searcher;
pub mod walker;

pub use cli::*;
pub use config::*;
pub use matcher::*;
pub use output::*;
pub use parser::*;
pub use replacer::*;
pub use searcher::*;
pub use walker::*;

use anyhow::Result;

/// Code-Grep のメインエラー型
#[derive(thiserror::Error, Debug)]
pub enum CodeGrepError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("FancyRegex error: {0}")]
    FancyRegex(#[from] fancy_regex::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Parser error: {0}")]
    Parser(String),
}

pub type CodeGrepResult<T> = Result<T, CodeGrepError>;