//! Creator-side editable document models. The Viewer never consumes these.

use std::collections::HashSet;

use ag_schema::{
    ContentBlock, CustomSchema, Document, FieldValues, Identity, RequestedRelease, SchemaError,
    StableId, Terminology, Validate,
};
use serde::{Deserialize, Serialize};

pub const WORKSPACE_FORMAT: &str = "afterglow-workspace";
pub const CONTRIBUTOR_FORMAT: &str = "afterglow-contributor";
pub const ENTRY_FORMAT: &str = "afterglow-entry";
pub const TERMINOLOGY_FORMAT: &str = "afterglow-terminology";
pub const SCHEMA_FORMAT: &str = "afterglow-schema";
pub const RELEASE_FORMAT: &str = "afterglow-release";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContributorStatus {
    Empty,
    Draft,
    Submitted,
    NeedsReview,
    Approved,
    LockedForBuild,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contributor {
    pub project_id: StableId,
    pub contributor_id: StableId,
    pub status: ContributorStatus,
    pub fields: FieldValues,
}

impl Validate for Contributor {
    fn validate(&self) -> Result<(), SchemaError> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub project_id: StableId,
    pub entry_id: StableId,
    pub contributor_id: StableId,
    pub title: String,
    pub blocks: Vec<ContentBlock>,
}

impl Validate for Entry {
    fn validate(&self) -> Result<(), SchemaError> {
        // Empty drafts are valid; completeness/build readiness is separate.
        if self.blocks.len() > 10_000 {
            return Err(SchemaError::Invalid("content block count"));
        }
        for block in &self.blocks {
            block.validate()?;
        }
        Ok(())
    }
}

/// Aggregate in-memory model, serialized as a versioned transport document.
/// Filesystem autosave, snapshots and contributor package import follow later.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub project_id: StableId,
    pub identity: Identity,
    pub locale: String,
    pub terminology: Document<Terminology>,
    pub schema: Document<CustomSchema>,
    pub release: Document<RequestedRelease>,
    pub contributors: Vec<Document<Contributor>>,
    pub entries: Vec<Document<Entry>>,
}

impl Validate for Project {
    fn validate(&self) -> Result<(), SchemaError> {
        self.identity.validate()?;
        if self.locale.trim().is_empty() {
            return Err(SchemaError::Invalid("project locale"));
        }
        self.terminology.validate_as(TERMINOLOGY_FORMAT)?;
        self.schema.validate_as(SCHEMA_FORMAT)?;
        self.release.validate_as(RELEASE_FORMAT)?;
        if self.contributors.len() > 100_000 || self.entries.len() > 100_000 {
            return Err(SchemaError::Invalid("project record count"));
        }
        let mut contributors = HashSet::new();
        for contributor in &self.contributors {
            contributor.validate_as(CONTRIBUTOR_FORMAT)?;
            let contributor = &contributor.data;
            if contributor.project_id != self.project_id
                || !contributors.insert(contributor.contributor_id)
            {
                return Err(SchemaError::Invalid("contributor project/identity"));
            }
            match contributor.status {
                ContributorStatus::Empty | ContributorStatus::Draft => self
                    .schema
                    .data
                    .validate_partial_values(&contributor.fields)?,
                _ => self.schema.data.validate_values(&contributor.fields)?,
            }
        }
        let mut entries = HashSet::new();
        for entry in &self.entries {
            entry.validate_as(ENTRY_FORMAT)?;
            let entry = &entry.data;
            if entry.project_id != self.project_id
                || !entries.insert(entry.entry_id)
                || !contributors.contains(&entry.contributor_id)
            {
                return Err(SchemaError::Invalid("entry project/identity/contributor"));
            }
        }
        Ok(())
    }
}

pub type ProjectDocument = Document<Project>;
