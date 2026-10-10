//! Domain-neutral formats shared by Builder and Viewer. No release capability.

mod content;
mod fields;
mod identifiers;
mod manifest;
mod version;

pub use content::*;
pub use fields::*;
pub use identifiers::*;
pub use manifest::*;
pub use version::*;

#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("invalid {0}")]
    Invalid(&'static str),
    #[error("unsupported format: {0}")]
    Format(String),
    #[error("unsupported format version {0}")]
    Version(u16),
    #[error("reader version {required} required; current reader is {current}")]
    Reader { required: u16, current: u16 },
    #[error("JSON exceeds the {0}-byte input limit")]
    TooLarge(usize),
    #[error("JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
}

pub trait Validate {
    fn validate(&self) -> Result<(), SchemaError>;
}

pub(crate) fn nonempty(value: &str, label: &'static str) -> Result<(), SchemaError> {
    if value.trim().is_empty() {
        Err(SchemaError::Invalid(label))
    } else {
        Ok(())
    }
}
