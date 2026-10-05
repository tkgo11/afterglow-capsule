use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{SchemaError, StableId, Validate, nonempty};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terminology {
    pub contributor_singular: String,
    pub contributor_plural: String,
    pub recipient_singular: String,
    pub recipient_plural: String,
    pub entry_singular: String,
    pub entry_plural: String,
    pub role_label: String,
    pub archive_label: String,
    pub open_action: String,
    pub locked_label: String,
}

impl Default for Terminology {
    fn default() -> Self {
        Self {
            contributor_singular: "Contributor".into(),
            contributor_plural: "Contributors".into(),
            recipient_singular: "Recipient".into(),
            recipient_plural: "Recipients".into(),
            entry_singular: "Message".into(),
            entry_plural: "Messages".into(),
            role_label: "Role".into(),
            archive_label: "Archive".into(),
            open_action: "Open".into(),
            locked_label: "Locked".into(),
        }
    }
}

impl Validate for Terminology {
    fn validate(&self) -> Result<(), SchemaError> {
        for term in [
            &self.contributor_singular,
            &self.contributor_plural,
            &self.recipient_singular,
            &self.recipient_plural,
            &self.entry_singular,
            &self.entry_plural,
            &self.role_label,
            &self.archive_label,
            &self.open_action,
            &self.locked_label,
        ] {
            nonempty(term, "terminology token")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    BuilderOnly,
    PublicPreRelease,
    PublicPostRelease,
    PrivateEncrypted,
    MetadataOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldType {
    ShortText,
    LongText,
    Number,
    Date,
    DateRange,
    SingleSelect,
    MultiSelect,
    Boolean,
    Image,
    TagList,
    Url,
    HiddenId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub id: String,
    pub label: String,
    pub field_type: FieldType,
    pub visibility: Visibility,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub choices: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomSchema {
    pub fields: Vec<FieldDefinition>,
}

pub type FieldValues = BTreeMap<String, Value>;

impl Validate for CustomSchema {
    fn validate(&self) -> Result<(), SchemaError> {
        if self.fields.len() > 1024 {
            return Err(SchemaError::Invalid("schema field count"));
        }
        let mut ids = HashSet::new();
        for field in &self.fields {
            nonempty(&field.id, "field identity")?;
            nonempty(&field.label, "field label")?;
            if !ids.insert(&field.id) {
                return Err(SchemaError::Invalid("duplicate field identity"));
            }
            let selectable = matches!(
                field.field_type,
                FieldType::SingleSelect | FieldType::MultiSelect
            );
            if selectable == field.choices.is_empty() {
                return Err(SchemaError::Invalid("select choices"));
            }
            let mut choices = HashSet::new();
            for choice in &field.choices {
                nonempty(choice, "select choice")?;
                if !choices.insert(choice) {
                    return Err(SchemaError::Invalid("duplicate select choice"));
                }
            }
        }
        Ok(())
    }
}

impl CustomSchema {
    pub fn validate_values(&self, values: &FieldValues) -> Result<(), SchemaError> {
        self.validate_values_with_required(values, true)
    }

    /// Drafts may omit required values; supplied values must still be well typed.
    pub fn validate_partial_values(&self, values: &FieldValues) -> Result<(), SchemaError> {
        self.validate_values_with_required(values, false)
    }

    fn validate_values_with_required(
        &self,
        values: &FieldValues,
        require_complete: bool,
    ) -> Result<(), SchemaError> {
        self.validate()?;
        if values
            .keys()
            .any(|id| !self.fields.iter().any(|f| &f.id == id))
        {
            return Err(SchemaError::Invalid("unknown field value"));
        }
        for field in &self.fields {
            let value = values.get(&field.id).filter(|v| !v.is_null());
            let Some(value) = value else {
                if field.required && require_complete {
                    return Err(SchemaError::Invalid("missing required field"));
                }
                continue;
            };
            validate_field_value(field, value)?;
        }
        Ok(())
    }
}

fn validate_field_value(field: &FieldDefinition, value: &Value) -> Result<(), SchemaError> {
    let strings = || {
        value
            .as_array()
            .is_some_and(|items| items.iter().all(|v| v.as_str().is_some()))
    };
    let date = |v: &Value| {
        v.as_str()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    };
    let valid = match field.field_type {
        FieldType::ShortText | FieldType::LongText => value
            .as_str()
            .is_some_and(|s| !field.required || !s.trim().is_empty()),
        FieldType::Number => value.is_number(),
        FieldType::Date => date(value).is_some(),
        FieldType::DateRange => value.as_object().is_some_and(|range| {
            match (
                range.get("start").and_then(date),
                range.get("end").and_then(date),
            ) {
                (Some(start), Some(end)) => start <= end,
                _ => false,
            }
        }),
        FieldType::SingleSelect => value
            .as_str()
            .is_some_and(|s| field.choices.iter().any(|choice| choice == s)),
        FieldType::MultiSelect => {
            strings()
                && value.as_array().is_some_and(|items| {
                    let mut seen = HashSet::new();
                    items.iter().all(|v| {
                        v.as_str()
                            .is_some_and(|s| field.choices.iter().any(|c| c == s) && seen.insert(s))
                    })
                })
        }
        FieldType::Boolean => value.is_boolean(),
        FieldType::Image | FieldType::HiddenId => {
            value.as_str().is_some_and(|s| StableId::parse(s).is_ok())
        }
        FieldType::TagList => strings(),
        FieldType::Url => value.as_str().is_some_and(valid_external_url),
    };
    if valid {
        Ok(())
    } else {
        Err(SchemaError::Invalid("field value"))
    }
}

pub fn valid_external_url(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| {
        matches!(url.scheme(), "https" | "http")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}
