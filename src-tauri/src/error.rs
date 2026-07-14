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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_db() {
        let err = Error::Db(rusqlite::Error::InvalidParameterName("test".into()));
        assert!(err.to_string().contains("Database error"));
    }

    #[test]
    fn test_error_display_io() {
        let err = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(err.to_string().contains("IO error"));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_error_display_audio() {
        let err = Error::Audio("device busy".into());
        assert_eq!(err.to_string(), "Audio engine error: device busy");
    }

    #[test]
    fn test_error_display_scan() {
        let err = Error::Scan("no files found".into());
        assert_eq!(err.to_string(), "Scan error: no files found");
    }

    #[test]
    fn test_error_display_migration() {
        let err = Error::Migration("version mismatch".into());
        assert_eq!(err.to_string(), "Migration error: version mismatch");
    }

    #[test]
    fn test_error_display_unknown() {
        let err = Error::Unknown("something went wrong".into());
        assert_eq!(err.to_string(), "Unknown error: something went wrong");
    }

    #[test]
    fn test_error_serialize() {
        let err = Error::Unknown("test error".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"Unknown error: test error\"");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
