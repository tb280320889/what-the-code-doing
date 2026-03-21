use thiserror::Error;

#[derive(Error, Debug)]
pub enum WtcdError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Parse error in {file}: {reason}")]
    ParseError { file: String, reason: String },

    #[error("Scope error: {0}")]
    ScopeError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlError(String),

    #[error("No language adapter found for: {ext}")]
    UnsupportedLanguage { ext: String },
}

pub type Result<T> = std::result::Result<T, WtcdError>;
