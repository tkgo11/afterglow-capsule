//! Prepared release-owned Viewer state. UI cannot set the state or obtain CEK.
//! No preview controls or persisted secrets; no production integration yet.

use std::{
    collections::{HashSet, VecDeque},
    time::{Duration, Instant},
};

use ag_prepared_capsule::LoadedCapsule;
use ag_prepared_object_crypto::{ContentKey, decrypt};
use ag_prepared_object_store::{Binding, Limits};
use ag_prepared_timelock::{FetchOutcome, QuicknetEngine, TimelockEngine};
use ag_schema::{ContentBlock, Document, ObjectClass, SchemaError, StableId, Validate};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

pub const ARCHIVE_FORMAT: &str = "afterglow-private-archive";
pub const ENTRY_FORMAT: &str = "afterglow-private-entry";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Boot,
    PreRelease,
    ReleaseMaterialCheck,
    Unlocking,
    Authenticating,
    ReadyForCeremony,
    Ceremony,
    PostReleaseHome,
    EntryDetail(StableId),
    MediaViewer {
        entry_id: StableId,
        object_id: StableId,
    },
    LockedError(Failure),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    ReleaseIntegrity,
    ObjectIntegrity,
    PrivateFormat,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("action is unavailable in the current Viewer state")]
    State,
    #[error("release retry is rate-limited")]
    RateLimited,
    #[error("entry/media identity is unavailable or not referenced by this entry")]
    Navigation,
    #[error("protected content remains locked: {0:?}")]
    Locked(Failure),
    #[error(transparent)]
    Capsule(#[from] ag_prepared_capsule::CapsuleError),
    #[error(transparent)]
    Timelock(#[from] ag_prepared_timelock::TimelockError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContributorIndex {
    pub contributor_id: StableId,
    pub profile_object_id: StableId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryIndex {
    pub entry_id: StableId,
    pub contributor_id: StableId,
    pub object_id: StableId,
}

/// Encrypted required index contains identities/references, never Creator state.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveIndex {
    pub project_id: StableId,
    pub build_id: StableId,
    pub contributors: Vec<ContributorIndex>,
    pub entries: Vec<EntryIndex>,
}

impl Validate for ArchiveIndex {
    fn validate(&self) -> Result<(), SchemaError> {
        if self.contributors.len() > 100_000
            || self.entries.len() > 100_000
            || self.entries.is_empty()
        {
            return Err(SchemaError::Invalid("archive record count"));
        }
        let mut contributors = HashSet::new();
        let mut entries = HashSet::new();
        let mut objects = HashSet::new();
        for contributor in &self.contributors {
            if !contributors.insert(contributor.contributor_id)
                || !objects.insert(contributor.profile_object_id)
            {
                return Err(SchemaError::Invalid("archive contributor/object identity"));
            }
        }
        for entry in &self.entries {
            if !contributors.contains(&entry.contributor_id)
                || !entries.insert(entry.entry_id)
                || !objects.insert(entry.object_id)
            {
                return Err(SchemaError::Invalid(
                    "archive entry/contributor/object identity",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct EntryContent {
    pub title: String,
    pub blocks: Vec<ContentBlock>,
}

impl Validate for EntryContent {
    fn validate(&self) -> Result<(), SchemaError> {
        if self.blocks.len() > 10_000 {
            return Err(SchemaError::Invalid("entry block count"));
        }
        for block in &self.blocks {
            block.validate()?;
        }
        Ok(())
    }
}

/// Borrow for rendering; redact Debug and clear owned text buffers on drop.
pub struct SecretEntry(EntryContent);

impl SecretEntry {
    pub fn content(&self) -> &EntryContent {
        &self.0
    }
}

impl std::fmt::Debug for SecretEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretEntry([REDACTED])")
    }
}

impl Drop for EntryContent {
    fn drop(&mut self) {
        self.title.zeroize();
        for block in &mut self.blocks {
            match block {
                ContentBlock::Heading { content, .. }
                | ContentBlock::Paragraph { content }
                | ContentBlock::Quote { content }
                | ContentBlock::Signature { content }
                | ContentBlock::Caption { content }
                | ContentBlock::Callout { content }
                | ContentBlock::Credits { content } => clear_text(content),
                ContentBlock::Image { alt, .. } => alt.zeroize(),
                ContentBlock::ImagePair { alt, .. } => alt.iter_mut().for_each(Zeroize::zeroize),
                ContentBlock::Gallery { alt, .. } => alt.iter_mut().for_each(Zeroize::zeroize),
                ContentBlock::Timeline { items } => items
                    .iter_mut()
                    .for_each(|item| clear_text(&mut item.content)),
                ContentBlock::Link { label, url } => {
                    label.zeroize();
                    url.zeroize();
                }
                ContentBlock::Audio { caption, .. } => clear_text(caption),
                _ => {}
            }
        }
    }
}

fn clear_text(text: &mut ag_schema::RichText) {
    for inline in text {
        if let ag_schema::Inline::Text { text, link, .. } = inline {
            text.zeroize();
            if let Some(link) = link {
                link.zeroize();
            }
        }
    }
}

pub struct ViewerSession<'a> {
    capsule: LoadedCapsule<'a>,
    engine: QuicknetEngine,
    state: State,
    cek: Option<ContentKey>,
    archive: Option<ArchiveIndex>,
    ceremony_complete: bool,
    last_request: Option<Instant>,
    history: VecDeque<State>,
}

impl<'a> ViewerSession<'a> {
    pub fn load(
        bytes: &'a [u8],
        viewer: &semver::Version,
        limits: Limits,
    ) -> Result<Self, SessionError> {
        let capsule = LoadedCapsule::parse(bytes, viewer, limits)?;
        let engine = QuicknetEngine::new(capsule.manifest().release.clone())?;
        Ok(Self {
            capsule,
            engine,
            state: State::Boot,
            cek: None,
            archive: None,
            ceremony_complete: false,
            last_request: None,
            history: VecDeque::from([State::Boot]),
        })
    }

    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn history(&self) -> &VecDeque<State> {
        &self.history
    }
    pub fn manifest(&self) -> &ag_schema::Manifest {
        self.capsule.manifest()
    }

    fn transition(&mut self, state: State) {
        self.state = state.clone();
        if self.history.len() == 64 {
            self.history.pop_front();
        }
        self.history.push_back(state);
    }

    fn fail<T>(&mut self, failure: Failure) -> Result<T, SessionError> {
        self.cek = None;
        self.archive = None;
        self.ceremony_complete = false;
        self.transition(State::LockedError(failure));
        Err(SessionError::Locked(failure))
    }

    pub fn start(&mut self) -> Result<(), SessionError> {
        if self.state != State::Boot {
            return Err(SessionError::State);
        }
        self.transition(State::PreRelease);
        Ok(())
    }

    pub fn begin_release_check(&mut self) -> Result<(), SessionError> {
        if self.state != State::PreRelease {
            return Err(SessionError::State);
        }
        let now = Instant::now();
        if self
            .last_request
            .is_some_and(|last| now.duration_since(last) < Duration::from_secs(2))
        {
            return Err(SessionError::RateLimited);
        }
        self.last_request = Some(now);
        self.transition(State::ReleaseMaterialCheck);
        Ok(())
    }

    pub fn apply_release_result(&mut self, outcome: FetchOutcome) -> Result<(), SessionError> {
        if self.state != State::ReleaseMaterialCheck {
            return Err(SessionError::State);
        }
        let beacon = match outcome {
            FetchOutcome::Waiting(_) => {
                self.transition(State::PreRelease);
                return Ok(());
            }
            FetchOutcome::Verified(beacon) => beacon,
        };
        self.transition(State::Unlocking);
        let key = match self.engine.unlock_cek(self.capsule.envelope(), &beacon) {
            Ok(key) => key,
            Err(_) => return self.fail(Failure::ReleaseIntegrity),
        };
        self.transition(State::Authenticating);
        let id = self.capsule.manifest().private_manifest_id;
        let bytes = match self.capsule.object(id) {
            Ok(bytes) => bytes,
            Err(_) => return self.fail(Failure::PrivateFormat),
        };
        let plaintext = match decrypt(&key, self.binding(id), bytes, self.capsule.limits()) {
            Ok(bytes) => bytes,
            Err(_) => return self.fail(Failure::ObjectIntegrity),
        };
        let index = match Document::<ArchiveIndex>::from_json(&plaintext, ARCHIVE_FORMAT) {
            Ok(index) => index.data,
            Err(_) => return self.fail(Failure::PrivateFormat),
        };
        if index.project_id != self.capsule.manifest().project_id
            || index.build_id != self.capsule.manifest().build_id
            || index
                .contributors
                .iter()
                .any(|c| !self.is_class(c.profile_object_id, ObjectClass::PrivateJson))
            || index
                .entries
                .iter()
                .any(|e| !self.is_class(e.object_id, ObjectClass::PrivateJson))
        {
            return self.fail(Failure::PrivateFormat);
        }
        self.archive = Some(index);
        self.cek = Some(key);
        self.transition(State::ReadyForCeremony);
        Ok(())
    }

    fn binding(&self, object_id: StableId) -> Binding {
        Binding {
            project_id: self.capsule.manifest().project_id,
            build_id: self.capsule.manifest().build_id,
            object_id,
        }
    }

    fn is_class(&self, id: StableId, class: ObjectClass) -> bool {
        self.capsule
            .metadata(id)
            .is_some_and(|object| object.class == class)
    }

    pub fn start_ceremony(&mut self) -> Result<(), SessionError> {
        if self.state != State::ReadyForCeremony {
            return Err(SessionError::State);
        }
        self.transition(State::Ceremony);
        Ok(())
    }

    pub fn complete_ceremony(&mut self) -> Result<(), SessionError> {
        if self.state != State::Ceremony {
            return Err(SessionError::State);
        }
        self.ceremony_complete = true;
        Ok(())
    }

    pub fn enter_archive(&mut self) -> Result<(), SessionError> {
        if self.state != State::Ceremony || !self.ceremony_complete {
            return Err(SessionError::State);
        }
        self.transition(State::PostReleaseHome);
        Ok(())
    }

    pub fn archive_index(&self) -> Result<&ArchiveIndex, SessionError> {
        if !matches!(
            self.state,
            State::PostReleaseHome | State::EntryDetail(_) | State::MediaViewer { .. }
        ) {
            return Err(SessionError::State);
        }
        self.archive.as_ref().ok_or(SessionError::State)
    }

    pub fn open_entry(&mut self, id: StableId) -> Result<(), SessionError> {
        if !self
            .archive_index()?
            .entries
            .iter()
            .any(|e| e.entry_id == id)
        {
            return Err(SessionError::Navigation);
        }
        self.transition(State::EntryDetail(id));
        Ok(())
    }

    pub fn read_entry(&mut self) -> Result<SecretEntry, SessionError> {
        let id = match self.state {
            State::EntryDetail(id) | State::MediaViewer { entry_id: id, .. } => id,
            _ => return Err(SessionError::State),
        };
        let object = self
            .archive_index()?
            .entries
            .iter()
            .find(|e| e.entry_id == id)
            .ok_or(SessionError::Navigation)?
            .object_id;
        let plaintext = self.decrypt_object(object)?;
        let entry = match Document::<EntryContent>::from_json(&plaintext, ENTRY_FORMAT) {
            Ok(entry) => entry.data,
            Err(_) => return self.fail(Failure::PrivateFormat),
        };
        Ok(SecretEntry(entry))
    }

    fn decrypt_object(&mut self, id: StableId) -> Result<Zeroizing<Vec<u8>>, SessionError> {
        self.archive_index()?;
        let key = self.cek.as_ref().ok_or(SessionError::State)?;
        let bytes = self.capsule.object(id)?;
        match decrypt(key, self.binding(id), bytes, self.capsule.limits()) {
            Ok(bytes) => Ok(bytes),
            Err(_) => self.fail(Failure::ObjectIntegrity),
        }
    }

    pub fn open_media(&mut self, id: StableId) -> Result<(), SessionError> {
        let entry_id = match self.state {
            State::EntryDetail(id) => id,
            _ => return Err(SessionError::State),
        };
        let entry = self.read_entry()?;
        let referenced = entry.content().blocks.iter().any(|block| match block {
            ContentBlock::Image { object_id, .. } => *object_id == id,
            ContentBlock::ImagePair { object_ids, .. } => object_ids.contains(&id),
            ContentBlock::Gallery { object_ids, .. } => object_ids.contains(&id),
            _ => false,
        });
        if !referenced
            || !(self.is_class(id, ObjectClass::PrivateImage)
                || self.is_class(id, ObjectClass::PublicImage))
        {
            return Err(SessionError::Navigation);
        }
        self.transition(State::MediaViewer {
            entry_id,
            object_id: id,
        });
        Ok(())
    }

    pub fn read_media(&mut self) -> Result<Zeroizing<Vec<u8>>, SessionError> {
        let State::MediaViewer { object_id, .. } = self.state else {
            return Err(SessionError::State);
        };
        if self.is_class(object_id, ObjectClass::PrivateImage) {
            self.decrypt_object(object_id)
        } else {
            Ok(Zeroizing::new(self.capsule.object(object_id)?.to_vec()))
        }
    }

    pub fn back(&mut self) -> Result<(), SessionError> {
        let next = match self.state {
            State::MediaViewer { entry_id, .. } => State::EntryDetail(entry_id),
            State::EntryDetail(_) => State::PostReleaseHome,
            State::PostReleaseHome => State::PostReleaseHome,
            _ => return Err(SessionError::State),
        };
        self.transition(next);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
