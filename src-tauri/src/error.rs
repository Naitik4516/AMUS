use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Database pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Metadata error: {0}")]
    Metadata(#[from] anyhow::Error),

    #[error("Audio engine error: {0}")]
    Audio(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Scan error: {0}")]
    Scan(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
