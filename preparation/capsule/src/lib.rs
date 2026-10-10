//! Isolated capsule serialization/validation, with no PE or signing adoption.

use std::collections::{BTreeMap, BTreeSet};

use ag_capsule::{CapsuleHeader, FORMAT_VERSION, HEADER_SIZE, Section, SectionKind};
use ag_prepared_object_crypto::{BuildEncryptor, CompressionPolicy, CryptoError};
use ag_prepared_object_store::{Binding, EncryptedObject, Limits};
use ag_prepared_timelock::{QuicknetEngine, TimelockEngine};
use ag_schema::{
    Manifest, ObjectClass, ObjectMetadata, ObjectProtection, StableId, Validate,
    validate_object_store_bounds,
};
use sha2::{Digest, Sha256};

const DIGEST_MAGIC: [u8; 8] = *b"AGDIG1\0\0";
const DIGEST_HEADER: usize = 16;
const DIGEST_RECORD: usize = 40;
const DIGEST_SIZE: usize = DIGEST_HEADER + 5 * DIGEST_RECORD;
const KINDS: [SectionKind; 4] = [
    SectionKind::Manifest,
    SectionKind::PublicStore,
    SectionKind::PrivateStore,
    SectionKind::Timelock,
];

#[derive(Debug, thiserror::Error)]
pub enum CapsuleError {
    #[error(transparent)]
    Header(#[from] ag_capsule::CapsuleError),
    #[error(transparent)]
    Schema(#[from] ag_schema::SchemaError),
    #[error(transparent)]
    Object(#[from] ag_prepared_object_store::StoreError),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error(transparent)]
    Timelock(#[from] ag_prepared_timelock::TimelockError),
    #[error("capsule layout is not canonical or exceeds the configured limit")]
    Layout,
    #[error("capsule section or public-object digest verification failed")]
    Digest,
    #[error("encrypted-object metadata does not match the public manifest")]
    Metadata,
    #[error("known private plaintext or CEK was found in capsule bytes")]
    Plaintext,
}

/// Fully validated structure and digests, not release authorization.
pub struct LoadedCapsule<'a> {
    manifest: Manifest,
    object_index: BTreeMap<StableId, usize>,
    public_store: &'a [u8],
    private_store: &'a [u8],
    envelope: &'a [u8],
    limits: Limits,
}

impl<'a> LoadedCapsule<'a> {
    pub fn parse(
        bytes: &'a [u8],
        viewer: &semver::Version,
        limits: Limits,
    ) -> Result<Self, CapsuleError> {
        let header = CapsuleHeader::parse_with_limit(bytes, limits.max_store_bytes)?;
        let mut cursor = HEADER_SIZE as u64;
        for section in header.sections {
            if section.length == 0 {
                continue;
            }
            if section.offset != cursor {
                return Err(CapsuleError::Layout);
            }
            cursor = cursor
                .checked_add(section.length)
                .ok_or(CapsuleError::Layout)?;
        }
        if cursor != bytes.len() as u64 {
            return Err(CapsuleError::Layout);
        }
        let digests = header.section(bytes, SectionKind::DigestTable, limits.max_store_bytes)?;
        if digests.len() != DIGEST_SIZE
            || digests[..8] != DIGEST_MAGIC
            || digests[8..10] != 1_u16.to_le_bytes()
            || digests[10..12] != 1_u16.to_le_bytes()
            || digests[12..16] != 5_u32.to_le_bytes()
        {
            return Err(CapsuleError::Digest);
        }
        for index in 0..5 {
            let record = &digests[DIGEST_HEADER + index * DIGEST_RECORD
                ..DIGEST_HEADER + (index + 1) * DIGEST_RECORD];
            let source = if index == 0 {
                &bytes[..HEADER_SIZE]
            } else {
                header.section(bytes, KINDS[index - 1], limits.max_store_bytes)?
            };
            if record[0] != index as u8
                || record[1..8].iter().any(|b| *b != 0)
                || record[8..] != Sha256::digest(source)[..]
            {
                return Err(CapsuleError::Digest);
            }
        }
        let manifest = Manifest::from_json(
            header.section(bytes, SectionKind::Manifest, limits.max_store_bytes)?,
            viewer,
        )?;
        let public_store =
            header.section(bytes, SectionKind::PublicStore, limits.max_store_bytes)?;
        let private_store =
            header.section(bytes, SectionKind::PrivateStore, limits.max_store_bytes)?;
        let envelope = header.section(bytes, SectionKind::Timelock, limits.max_store_bytes)?;
        validate_object_store_bounds(
            &manifest.objects,
            public_store.len() as u64,
            private_store.len() as u64,
        )?;
        let object_index = manifest
            .objects
            .iter()
            .enumerate()
            .map(|(index, object)| (object.object_id, index))
            .collect();
        let capsule = Self {
            manifest,
            object_index,
            public_store,
            private_store,
            envelope,
            limits,
        };
        let mut public_end = 0;
        let mut private_end = 0;
        let mut ordered: Vec<_> = capsule.manifest.objects.iter().collect();
        ordered.sort_by_key(|o| (o.class.is_private(), o.offset));
        for metadata in ordered {
            let bytes = capsule.object(metadata.object_id)?;
            let end = if metadata.class.is_private() {
                &mut private_end
            } else {
                &mut public_end
            };
            if metadata.offset != *end {
                return Err(CapsuleError::Layout);
            }
            *end = metadata.offset + metadata.length;
            match &metadata.protection {
                ObjectProtection::Public { sha256 } => {
                    if metadata.plaintext_length != metadata.length
                        || metadata.compression != ag_schema::Compression::None
                        || *sha256 != hex::encode(Sha256::digest(bytes))
                    {
                        return Err(CapsuleError::Digest);
                    }
                }
                ObjectProtection::Private {
                    nonce_prefix,
                    chunk_size,
                    chunk_count,
                } => {
                    let object = EncryptedObject::parse(bytes, limits)?;
                    let compression = match object.header.compression {
                        ag_prepared_object_store::Compression::None => ag_schema::Compression::None,
                        ag_prepared_object_store::Compression::Zstandard => {
                            ag_schema::Compression::Zstandard
                        }
                    };
                    if object.header.binding
                        != (Binding {
                            project_id: capsule.manifest.project_id,
                            build_id: capsule.manifest.build_id,
                            object_id: metadata.object_id,
                        })
                        || object.header.plaintext_length != metadata.plaintext_length
                        || object.header.nonce_prefix != *nonce_prefix
                        || object.header.chunk_size != *chunk_size
                        || object.header.chunk_count != *chunk_count
                        || compression != metadata.compression
                    {
                        return Err(CapsuleError::Metadata);
                    }
                }
            }
        }
        if public_end != public_store.len() as u64 || private_end != private_store.len() as u64 {
            return Err(CapsuleError::Layout);
        }
        // The envelope must be pinned before the Viewer starts networking.
        QuicknetEngine::new(capsule.manifest.release.clone())?.validate_envelope(envelope)?;
        Ok(capsule)
    }

    pub fn object(&self, id: StableId) -> Result<&'a [u8], CapsuleError> {
        let object = self.metadata(id).ok_or(CapsuleError::Metadata)?;
        let source = if object.class.is_private() {
            self.private_store
        } else {
            self.public_store
        };
        let start = usize::try_from(object.offset).map_err(|_| CapsuleError::Layout)?;
        let length = usize::try_from(object.length).map_err(|_| CapsuleError::Layout)?;
        source
            .get(start..start.checked_add(length).ok_or(CapsuleError::Layout)?)
            .ok_or(CapsuleError::Layout)
    }

    pub fn envelope(&self) -> &'a [u8] {
        self.envelope
    }
    pub fn limits(&self) -> Limits {
        self.limits
    }
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn metadata(&self, id: StableId) -> Option<&ObjectMetadata> {
        self.object_index
            .get(&id)
            .map(|index| &self.manifest.objects[*index])
    }
}

pub struct CapsuleAssembler {
    manifest: Manifest,
    encryptor: BuildEncryptor,
    engine: QuicknetEngine,
    public_store: Vec<u8>,
    private_store: Vec<u8>,
    limits: Limits,
    object_ids: BTreeSet<StableId>,
}

impl CapsuleAssembler {
    pub fn new(manifest: Manifest, limits: Limits) -> Result<Self, CapsuleError> {
        if !manifest.objects.is_empty() {
            return Err(CapsuleError::Metadata);
        }
        manifest.identity.validate()?;
        let engine = QuicknetEngine::new(manifest.release.clone())?;
        let encryptor = BuildEncryptor::new(manifest.project_id, manifest.build_id, limits)?;
        Ok(Self {
            manifest,
            encryptor,
            engine,
            public_store: Vec::new(),
            private_store: Vec::new(),
            limits,
            object_ids: BTreeSet::new(),
        })
    }

    fn reserve(&self, bytes: usize) -> Result<(), CapsuleError> {
        if (self.public_store.len() as u64)
            .checked_add(self.private_store.len() as u64)
            .and_then(|n| n.checked_add(bytes as u64))
            .is_none_or(|n| n > self.limits.max_store_bytes)
            || self.manifest.objects.len() >= 100_000
        {
            return Err(CapsuleError::Layout);
        }
        Ok(())
    }

    pub fn add_public(
        &mut self,
        id: StableId,
        class: ObjectClass,
        bytes: &[u8],
    ) -> Result<(), CapsuleError> {
        if class.is_private() || bytes.is_empty() || self.object_ids.contains(&id) {
            return Err(CapsuleError::Metadata);
        }
        self.reserve(bytes.len())?;
        self.object_ids.insert(id);
        self.manifest.objects.push(ObjectMetadata {
            object_id: id,
            class,
            offset: self.public_store.len() as u64,
            length: bytes.len() as u64,
            plaintext_length: bytes.len() as u64,
            compression: ag_schema::Compression::None,
            protection: ObjectProtection::Public {
                sha256: hex::encode(Sha256::digest(bytes)),
            },
        });
        self.public_store.extend_from_slice(bytes);
        Ok(())
    }

    pub fn add_private(
        &mut self,
        id: StableId,
        class: ObjectClass,
        bytes: &[u8],
        policy: CompressionPolicy,
    ) -> Result<(), CapsuleError> {
        if !class.is_private() || self.object_ids.contains(&id) {
            return Err(CapsuleError::Metadata);
        }
        self.reserve(0)?;
        let encrypted = self.encryptor.encrypt(id, bytes, policy)?;
        self.reserve(encrypted.len())?;
        let object = EncryptedObject::parse(&encrypted, self.limits)?;
        self.object_ids.insert(id);
        self.manifest.objects.push(ObjectMetadata {
            object_id: id,
            class,
            offset: self.private_store.len() as u64,
            length: encrypted.len() as u64,
            plaintext_length: bytes.len() as u64,
            compression: match object.header.compression {
                ag_prepared_object_store::Compression::None => ag_schema::Compression::None,
                ag_prepared_object_store::Compression::Zstandard => {
                    ag_schema::Compression::Zstandard
                }
            },
            protection: ObjectProtection::Private {
                nonce_prefix: object.header.nonce_prefix,
                chunk_count: object.header.chunk_count,
                chunk_size: object.header.chunk_size,
            },
        });
        self.private_store.extend_from_slice(&encrypted);
        Ok(())
    }

    /// CEK never leaves the assembler except into the interoperable envelope.
    /// Known plaintext checks are a safety net; the final EXE scan is still required.
    pub fn finish(
        self,
        viewer: &semver::Version,
        private_needles: &[&[u8]],
    ) -> Result<Vec<u8>, CapsuleError> {
        let envelope = self.encryptor.with_cek(|key| self.engine.lock_cek(key))?;
        let bytes = serialize(
            &self.manifest,
            &self.public_store,
            &self.private_store,
            &envelope,
            self.limits,
        )?;
        LoadedCapsule::parse(&bytes, viewer, self.limits)?;
        scan_plaintext(&bytes, private_needles)?;
        self.encryptor
            .with_cek(|key| key.with_secret(|secret| scan_plaintext(&bytes, &[secret])))?;
        Ok(bytes)
    }
}

pub fn scan_plaintext(bytes: &[u8], private_needles: &[&[u8]]) -> Result<(), CapsuleError> {
    for needle in private_needles {
        if needle.is_empty() {
            return Err(CapsuleError::Plaintext);
        }
        if bytes.windows(needle.len()).any(|window| window == *needle) {
            return Err(CapsuleError::Plaintext);
        }
    }
    Ok(())
}

pub fn serialize(
    manifest: &Manifest,
    public_store: &[u8],
    private_store: &[u8],
    envelope: &[u8],
    limits: Limits,
) -> Result<Vec<u8>, CapsuleError> {
    let manifest_bytes = manifest.to_json()?;
    let sources = [
        manifest_bytes.as_slice(),
        public_store,
        private_store,
        envelope,
    ];
    let mut header = CapsuleHeader {
        format_version: FORMAT_VERSION,
        minimum_reader_version: 1,
        flags: 0,
        sections: [Section::default(); 5],
    };
    let mut length = HEADER_SIZE as u64;
    for (index, source) in sources.iter().enumerate() {
        if source.is_empty() {
            continue;
        }
        header.sections[index] = Section {
            offset: length,
            length: source.len() as u64,
        };
        length = length
            .checked_add(source.len() as u64)
            .ok_or(CapsuleError::Layout)?;
    }
    header.sections[4] = Section {
        offset: length,
        length: DIGEST_SIZE as u64,
    };
    length = length
        .checked_add(DIGEST_SIZE as u64)
        .ok_or(CapsuleError::Layout)?;
    let header_bytes = header.encode(length, limits.max_store_bytes)?;
    let mut output = Vec::with_capacity(usize::try_from(length).map_err(|_| CapsuleError::Layout)?);
    output.extend_from_slice(&header_bytes);
    for source in sources {
        output.extend_from_slice(source);
    }
    output.extend_from_slice(&DIGEST_MAGIC);
    output.extend_from_slice(&1_u16.to_le_bytes());
    output.extend_from_slice(&1_u16.to_le_bytes());
    output.extend_from_slice(&5_u32.to_le_bytes());
    for (index, source) in std::iter::once(header_bytes.as_slice())
        .chain(sources)
        .enumerate()
    {
        output.push(index as u8);
        output.extend_from_slice(&[0; 7]);
        output.extend_from_slice(&Sha256::digest(source));
    }
    Ok(output)
}

#[cfg(test)]
mod tests;
