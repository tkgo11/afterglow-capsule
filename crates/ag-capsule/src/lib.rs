//! Fixed little-endian header and bounded structure validation (SPEC.md §19).
//! This parser grants no release capability and does not replace digest/AEAD checks.

pub const MAGIC: [u8; 8] = *b"AGCAPS1\0";
pub const FORMAT_NAME: &str = "afterglow-capsule";
pub const FORMAT_VERSION: u16 = 1;
pub const HEADER_SIZE: usize = 128;
pub const DEFAULT_MAX_CAPSULE_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionKind {
    Manifest,
    PublicStore,
    PrivateStore,
    Timelock,
    DigestTable,
}

impl SectionKind {
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Section {
    pub offset: u64,
    pub length: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapsuleHeader {
    pub format_version: u16,
    pub minimum_reader_version: u16,
    pub flags: u32,
    pub sections: [Section; 5],
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CapsuleError {
    #[error("capsule header is truncated")]
    Truncated,
    #[error("capsule magic is invalid")]
    Magic,
    #[error("unsupported capsule format/reader version")]
    Version,
    #[error("unsupported capsule header size")]
    HeaderSize,
    #[error("unknown capsule flags or nonzero reserved bytes")]
    Reserved,
    #[error("capsule exceeds the configured size limit")]
    TooLarge,
    #[error("capsule section extent is invalid or overflows")]
    Bounds,
    #[error("capsule sections overlap")]
    Overlap,
    #[error("required capsule section is missing")]
    MissingSection,
}

impl CapsuleHeader {
    pub fn parse(bytes: &[u8]) -> Result<Self, CapsuleError> {
        Self::parse_with_limit(bytes, DEFAULT_MAX_CAPSULE_BYTES)
    }

    /// Expert size overrides are explicit; they never relax structural validation.
    pub fn parse_with_limit(bytes: &[u8], max_bytes: u64) -> Result<Self, CapsuleError> {
        if bytes.len() < HEADER_SIZE {
            return Err(CapsuleError::Truncated);
        }
        if bytes[..8] != MAGIC {
            return Err(CapsuleError::Magic);
        }
        if u16::from_le_bytes(read(bytes, 10)?) != HEADER_SIZE as u16 {
            return Err(CapsuleError::HeaderSize);
        }
        if bytes[98..HEADER_SIZE].iter().any(|byte| *byte != 0) {
            return Err(CapsuleError::Reserved);
        }
        let mut header = Self {
            format_version: u16::from_le_bytes(read(bytes, 8)?),
            minimum_reader_version: u16::from_le_bytes(read(bytes, 96)?),
            flags: u32::from_le_bytes(read(bytes, 12)?),
            sections: [Section::default(); 5],
        };
        for (index, section) in header.sections.iter_mut().enumerate() {
            section.offset = u64::from_le_bytes(read(bytes, 16 + index * 16)?);
            section.length = u64::from_le_bytes(read(bytes, 24 + index * 16)?);
        }
        header.validate(bytes.len() as u64, max_bytes)?;
        Ok(header)
    }

    pub fn validate(&self, capsule_length: u64, max_bytes: u64) -> Result<(), CapsuleError> {
        if self.format_version != FORMAT_VERSION || self.minimum_reader_version != 1 {
            return Err(CapsuleError::Version);
        }
        if self.flags != 0 {
            return Err(CapsuleError::Reserved);
        }
        if capsule_length > max_bytes {
            return Err(CapsuleError::TooLarge);
        }
        if capsule_length < HEADER_SIZE as u64 {
            return Err(CapsuleError::Truncated);
        }
        let mut ranges = Vec::with_capacity(5);
        for (index, section) in self.sections.iter().enumerate() {
            if section.length == 0 {
                if index != SectionKind::PublicStore.index() {
                    return Err(CapsuleError::MissingSection);
                }
                if section.offset != 0 {
                    return Err(CapsuleError::Bounds);
                }
                continue;
            }
            let end = section
                .offset
                .checked_add(section.length)
                .ok_or(CapsuleError::Bounds)?;
            if section.offset < HEADER_SIZE as u64 || end > capsule_length {
                return Err(CapsuleError::Bounds);
            }
            ranges.push((section.offset, end));
        }
        ranges.sort_unstable();
        if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err(CapsuleError::Overlap);
        }
        Ok(())
    }

    /// Encodes the header only; full capsule serialization follows Phase 5.
    pub fn encode(
        &self,
        capsule_length: u64,
        max_bytes: u64,
    ) -> Result<[u8; HEADER_SIZE], CapsuleError> {
        self.validate(capsule_length, max_bytes)?;
        let mut bytes = [0; HEADER_SIZE];
        bytes[..8].copy_from_slice(&MAGIC);
        bytes[8..10].copy_from_slice(&self.format_version.to_le_bytes());
        bytes[10..12].copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        bytes[12..16].copy_from_slice(&self.flags.to_le_bytes());
        for (index, section) in self.sections.iter().enumerate() {
            bytes[16 + index * 16..24 + index * 16].copy_from_slice(&section.offset.to_le_bytes());
            bytes[24 + index * 16..32 + index * 16].copy_from_slice(&section.length.to_le_bytes());
        }
        bytes[96..98].copy_from_slice(&self.minimum_reader_version.to_le_bytes());
        Ok(bytes)
    }

    pub fn section<'a>(
        &self,
        bytes: &'a [u8],
        kind: SectionKind,
        max_bytes: u64,
    ) -> Result<&'a [u8], CapsuleError> {
        self.validate(bytes.len() as u64, max_bytes)?;
        let section = self.sections[kind.index()];
        let start = usize::try_from(section.offset).map_err(|_| CapsuleError::Bounds)?;
        let length = usize::try_from(section.length).map_err(|_| CapsuleError::Bounds)?;
        let end = start.checked_add(length).ok_or(CapsuleError::Bounds)?;
        bytes.get(start..end).ok_or(CapsuleError::Bounds)
    }
}

fn read<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N], CapsuleError> {
    let end = offset.checked_add(N).ok_or(CapsuleError::Bounds)?;
    bytes
        .get(offset..end)
        .ok_or(CapsuleError::Truncated)?
        .try_into()
        .map_err(|_| CapsuleError::Truncated)
}
