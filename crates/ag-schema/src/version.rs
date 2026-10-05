use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{SchemaError, Validate};

pub const READER_VERSION: u16 = 1;
pub const MAX_JSON_BYTES: usize = 16 * 1024 * 1024;

pub fn check_version(
    format_name: &str,
    expected_name: &str,
    format_version: u16,
    minimum_reader_version: u16,
) -> Result<(), SchemaError> {
    if format_name != expected_name {
        return Err(SchemaError::Format(format_name.to_owned()));
    }
    if format_version != 1 {
        return Err(SchemaError::Version(format_version));
    }
    if minimum_reader_version == 0 {
        return Err(SchemaError::Invalid("minimum reader version"));
    }
    if minimum_reader_version > READER_VERSION {
        return Err(SchemaError::Reader {
            required: minimum_reader_version,
            current: READER_VERSION,
        });
    }
    Ok(())
}

/// Builder documents use an explicit envelope. Capsule manifests have their own
/// top-level shape matching SPEC.md §15.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document<T> {
    pub format_name: String,
    pub format_version: u16,
    pub minimum_reader_version: u16,
    pub data: T,
}

impl<T: Validate> Document<T> {
    pub fn new(format_name: &str, data: T) -> Result<Self, SchemaError> {
        data.validate()?;
        Ok(Self {
            format_name: format_name.to_owned(),
            format_version: 1,
            minimum_reader_version: READER_VERSION,
            data,
        })
    }

    pub fn validate_as(&self, expected_name: &str) -> Result<(), SchemaError> {
        check_version(
            &self.format_name,
            expected_name,
            self.format_version,
            self.minimum_reader_version,
        )?;
        self.data.validate()
    }
}

impl<T: Validate + Serialize + DeserializeOwned> Document<T> {
    pub fn from_json(bytes: &[u8], expected_name: &str) -> Result<Self, SchemaError> {
        let document: Self = bounded_json(bytes)?;
        document.validate_as(expected_name)?;
        Ok(document)
    }

    pub fn to_json(&self, expected_name: &str) -> Result<Vec<u8>, SchemaError> {
        self.validate_as(expected_name)?;
        let bytes = serde_json::to_vec(self)?;
        if bytes.len() > MAX_JSON_BYTES {
            return Err(SchemaError::TooLarge(MAX_JSON_BYTES));
        }
        Ok(bytes)
    }
}

pub fn bounded_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, SchemaError> {
    if bytes.len() > MAX_JSON_BYTES {
        return Err(SchemaError::TooLarge(MAX_JSON_BYTES));
    }
    // serde_json's default recursion limit remains enabled.
    Ok(serde_json::from_slice(bytes)?)
}
