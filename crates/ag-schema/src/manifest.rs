use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{SchemaError, StableId, Validate, bounded_json, check_version, nonempty};

pub const MANIFEST_FORMAT: &str = "afterglow-project";
pub const DEFAULT_CHUNK_SIZE: u32 = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub introduction: String,
    #[serde(default)]
    pub creator_credit: String,
}

impl Validate for Identity {
    fn validate(&self) -> Result<(), SchemaError> {
        nonempty(&self.title, "project title")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CollectionLayout {
    Constellation,
    PortraitGrid,
    Timeline,
    Chapters,
    Carousel,
    SingleEntry,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub pre_release: String,
    pub post_release: CollectionLayout,
}

/// Schema validation checks the metadata, not cryptographic network identity.
/// Production trust/verification still requires the Spike A validated adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkProfile {
    pub profile_id: String,
    pub chain_hash: String,
    pub public_key: String,
    pub genesis_utc: DateTime<Utc>,
    pub period_seconds: u64,
    pub scheme_id: String,
    pub relays: Vec<String>,
}

impl Validate for NetworkProfile {
    fn validate(&self) -> Result<(), SchemaError> {
        nonempty(&self.profile_id, "network profile identity")?;
        nonempty(&self.scheme_id, "network scheme identity")?;
        if !valid_hex(&self.chain_hash, 32)
            || self.public_key.is_empty()
            || self.public_key.len() > 2048
            || !self.public_key.len().is_multiple_of(2)
            || !self.public_key.bytes().all(|b| b.is_ascii_hexdigit())
            || self.period_seconds == 0
        {
            return Err(SchemaError::Invalid("pinned network parameters"));
        }
        if !(2..=16).contains(&self.relays.len()) {
            return Err(SchemaError::Invalid("relay allowlist count"));
        }
        let mut relays = HashSet::new();
        for relay in &self.relays {
            let url = url::Url::parse(relay).map_err(|_| SchemaError::Invalid("relay URL"))?;
            if url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
                || !relays.insert(url)
            {
                return Err(SchemaError::Invalid("relay allowlist"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedRelease {
    pub utc: DateTime<Utc>,
    pub timezone: String,
    pub timelock_profile: String,
}

impl Validate for RequestedRelease {
    fn validate(&self) -> Result<(), SchemaError> {
        nonempty(&self.timelock_profile, "timelock profile")?;
        self.timezone
            .parse::<chrono_tz::Tz>()
            .map_err(|_| SchemaError::Invalid("IANA timezone"))?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinnedRelease {
    pub utc: DateTime<Utc>,
    pub timezone: String,
    pub target_round: u64,
    pub timelock_profile: String,
    pub network: NetworkProfile,
}

/// Exact integer ceiling, including fractional seconds. This is build-time
/// metadata logic; it never authorizes release or consults a runtime clock.
pub fn target_round(
    requested: DateTime<Utc>,
    genesis: DateTime<Utc>,
    period_seconds: u64,
) -> Result<u64, SchemaError> {
    if period_seconds == 0 || requested < genesis {
        return Err(SchemaError::Invalid(
            "release before genesis or zero period",
        ));
    }
    let nanos = |time: DateTime<Utc>| {
        i128::from(time.timestamp()) * 1_000_000_000 + i128::from(time.timestamp_subsec_nanos())
    };
    let delta = nanos(requested) - nanos(genesis);
    let period = i128::from(period_seconds) * 1_000_000_000;
    let round = delta / period + i128::from(delta % period != 0) + 1;
    u64::try_from(round).map_err(|_| SchemaError::Invalid("target round overflow"))
}

impl Validate for PinnedRelease {
    fn validate(&self) -> Result<(), SchemaError> {
        RequestedRelease {
            utc: self.utc,
            timezone: self.timezone.clone(),
            timelock_profile: self.timelock_profile.clone(),
        }
        .validate()?;
        self.network.validate()?;
        if self.timelock_profile != self.network.profile_id
            || self.target_round
                != target_round(
                    self.utc,
                    self.network.genesis_utc,
                    self.network.period_seconds,
                )?
        {
            return Err(SchemaError::Invalid("pinned target round/profile"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectClass {
    PublicJson,
    PublicImage,
    PublicAudio,
    PublicFont,
    PrivateJson,
    PrivateImage,
    PrivateAudio,
    PrivateVideo,
    PrivateBinary,
}

impl ObjectClass {
    pub fn is_private(self) -> bool {
        matches!(
            self,
            Self::PrivateJson
                | Self::PrivateImage
                | Self::PrivateAudio
                | Self::PrivateVideo
                | Self::PrivateBinary
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Compression {
    None,
    Zstandard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ObjectProtection {
    Public {
        sha256: String,
    },
    Private {
        nonce_prefix: [u8; 8],
        chunk_count: u32,
        chunk_size: u32,
    },
}

/// Offsets are relative to the object's public/private store, not the capsule.
/// Private titles, source filenames and other human metadata are not represented.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectMetadata {
    pub object_id: StableId,
    pub class: ObjectClass,
    pub offset: u64,
    pub length: u64,
    pub plaintext_length: u64,
    pub compression: Compression,
    pub protection: ObjectProtection,
}

impl Validate for ObjectMetadata {
    fn validate(&self) -> Result<(), SchemaError> {
        if self.length == 0 || self.offset.checked_add(self.length).is_none() {
            return Err(SchemaError::Invalid("object extent"));
        }
        match &self.protection {
            ObjectProtection::Public { sha256 }
                if !self.class.is_private() && valid_hex(sha256, 32) =>
            {
                Ok(())
            }
            ObjectProtection::Private {
                chunk_count,
                chunk_size,
                ..
            } if self.class.is_private() && *chunk_count > 0 && *chunk_size > 0 => Ok(()),
            _ => Err(SchemaError::Invalid("object protection/class")),
        }
    }
}

pub fn validate_object_table(objects: &[ObjectMetadata]) -> Result<(), SchemaError> {
    if objects.len() > 100_000 {
        return Err(SchemaError::Invalid("object count"));
    }
    let mut ids = HashSet::new();
    let mut nonces = HashSet::new();
    let mut public = Vec::new();
    let mut private = Vec::new();
    for object in objects {
        object.validate()?;
        if !ids.insert(object.object_id) {
            return Err(SchemaError::Invalid("duplicate object identity"));
        }
        if let ObjectProtection::Private { nonce_prefix, .. } = object.protection
            && !nonces.insert(nonce_prefix)
        {
            return Err(SchemaError::Invalid("duplicate nonce prefix"));
        }
        let store = if object.class.is_private() {
            &mut private
        } else {
            &mut public
        };
        store.push((object.offset, object.offset + object.length));
    }
    for store in [&mut public, &mut private] {
        store.sort_unstable();
        if store.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err(SchemaError::Invalid("overlapping objects"));
        }
    }
    Ok(())
}

fn valid_hex(value: &str, bytes: usize) -> bool {
    value.len() == bytes * 2 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

pub fn validate_object_store_bounds(
    objects: &[ObjectMetadata],
    public_store_length: u64,
    private_store_length: u64,
) -> Result<(), SchemaError> {
    validate_object_table(objects)?;
    for object in objects {
        let store_length = if object.class.is_private() {
            private_store_length
        } else {
            public_store_length
        };
        if object
            .offset
            .checked_add(object.length)
            .is_none_or(|end| end > store_length)
        {
            return Err(SchemaError::Invalid("object store bounds"));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub format_name: String,
    pub format_version: u16,
    pub minimum_reader_version: u16,
    pub project_id: StableId,
    pub build_id: StableId,
    pub viewer_min_version: semver::Version,
    pub identity: Identity,
    pub release: PinnedRelease,
    pub layout: Layout,
    pub theme_id: String,
    pub private_manifest_id: StableId,
    pub objects: Vec<ObjectMetadata>,
}

impl Validate for Manifest {
    fn validate(&self) -> Result<(), SchemaError> {
        check_version(
            &self.format_name,
            MANIFEST_FORMAT,
            self.format_version,
            self.minimum_reader_version,
        )?;
        self.identity.validate()?;
        self.release.validate()?;
        nonempty(&self.theme_id, "theme identity")?;
        // v1 supports only this declarative pre-release layout.
        if self.layout.pre_release != "countdown-default" {
            return Err(SchemaError::Invalid("pre-release layout"));
        }
        validate_object_table(&self.objects)?;
        if !self.objects.iter().any(|object| {
            object.object_id == self.private_manifest_id && object.class == ObjectClass::PrivateJson
        }) {
            return Err(SchemaError::Invalid("required private manifest object"));
        }
        Ok(())
    }
}

impl Manifest {
    pub fn from_json(bytes: &[u8], viewer_version: &semver::Version) -> Result<Self, SchemaError> {
        let manifest: Self = bounded_json(bytes)?;
        manifest.validate()?;
        if &manifest.viewer_min_version > viewer_version {
            return Err(SchemaError::Invalid("minimum Viewer version"));
        }
        Ok(manifest)
    }

    pub fn to_json(&self) -> Result<Vec<u8>, SchemaError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self)?;
        if bytes.len() > crate::MAX_JSON_BYTES {
            return Err(SchemaError::TooLarge(crate::MAX_JSON_BYTES));
        }
        Ok(bytes)
    }
}
