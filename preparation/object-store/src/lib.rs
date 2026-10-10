//! Bounded, borrowing encrypted-object reader. Preparation only; not in Viewer.

use ag_schema::StableId;

pub const MAGIC: [u8; 8] = *b"AGOBJ1\0\0";
pub const FORMAT_VERSION: u16 = 1;
pub const HEADER_SIZE: usize = 96;
pub const RECORD_SIZE: usize = 12;
pub const TAG_SIZE: usize = 16;
pub const DEFAULT_CHUNK_SIZE: u32 = 1024 * 1024;
pub const MAX_CHUNK_SIZE: u32 = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub project_id: StableId,
    pub build_id: StableId,
    pub object_id: StableId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Compression {
    None = 0,
    Zstandard = 1,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_store_bytes: u64,
    pub max_plaintext_bytes: u64,
    pub max_chunks: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_store_bytes: 1024 * 1024 * 1024,
            max_plaintext_bytes: 512 * 1024 * 1024,
            max_chunks: 65_536,
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum StoreError {
    #[error("unsupported encrypted-object format, reader or flags")]
    Format,
    #[error("encrypted object exceeds configured limits")]
    Limit,
    #[error("encrypted object is truncated or has invalid extents")]
    Bounds,
    #[error("encrypted-object chunk count, index or length is invalid")]
    Chunks,
    #[error("invalid encrypted-object identity")]
    Identity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub binding: Binding,
    pub compression: Compression,
    pub nonce_prefix: [u8; 8],
    pub chunk_size: u32,
    pub chunk_count: u32,
    pub plaintext_length: u64,
    pub encoded_length: u64,
}

impl Header {
    pub fn validate(&self, limits: Limits) -> Result<(), StoreError> {
        if self.chunk_size == 0 || self.chunk_size > MAX_CHUNK_SIZE {
            return Err(StoreError::Chunks);
        }
        if self.plaintext_length > limits.max_plaintext_bytes
            || self.encoded_length > limits.max_plaintext_bytes
            || self.chunk_count > limits.max_chunks
        {
            return Err(StoreError::Limit);
        }
        // u64 arithmetic also rejects an object requiring > u32::MAX chunks.
        let count = self
            .encoded_length
            .div_ceil(u64::from(self.chunk_size))
            .max(1);
        if count > u64::from(u32::MAX) || count != u64::from(self.chunk_count) {
            return Err(StoreError::Chunks);
        }
        if (self.compression == Compression::None && self.plaintext_length != self.encoded_length)
            || (self.compression == Compression::Zstandard
                && (self.encoded_length == 0 || self.encoded_length >= self.plaintext_length))
        {
            return Err(StoreError::Chunks);
        }
        Ok(())
    }

    pub fn encode(&self, limits: Limits) -> Result<[u8; HEADER_SIZE], StoreError> {
        self.validate(limits)?;
        let mut out = [0; HEADER_SIZE];
        out[..8].copy_from_slice(&MAGIC);
        out[8..10].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
        out[10..12].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
        out[12..14].copy_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        out[14] = self.compression as u8;
        out[16..32].copy_from_slice(self.binding.project_id.as_bytes());
        out[32..48].copy_from_slice(self.binding.build_id.as_bytes());
        out[48..64].copy_from_slice(self.binding.object_id.as_bytes());
        out[64..72].copy_from_slice(&self.nonce_prefix);
        out[72..76].copy_from_slice(&self.chunk_size.to_le_bytes());
        out[76..80].copy_from_slice(&self.chunk_count.to_le_bytes());
        out[80..88].copy_from_slice(&self.plaintext_length.to_le_bytes());
        out[88..96].copy_from_slice(&self.encoded_length.to_le_bytes());
        Ok(out)
    }

    pub fn parse(bytes: &[u8], limits: Limits) -> Result<Self, StoreError> {
        if bytes.len() < HEADER_SIZE {
            return Err(StoreError::Bounds);
        }
        if bytes[..8] != MAGIC
            || u16::from_le_bytes(array(bytes, 8)?) != FORMAT_VERSION
            || u16::from_le_bytes(array(bytes, 10)?) != FORMAT_VERSION
            || u16::from_le_bytes(array(bytes, 12)?) != HEADER_SIZE as u16
            || bytes[15] != 0
        {
            return Err(StoreError::Format);
        }
        let compression = match bytes[14] {
            0 => Compression::None,
            1 => Compression::Zstandard,
            _ => return Err(StoreError::Format),
        };
        let header = Self {
            binding: Binding {
                project_id: identity(bytes, 16)?,
                build_id: identity(bytes, 32)?,
                object_id: identity(bytes, 48)?,
            },
            compression,
            nonce_prefix: array(bytes, 64)?,
            chunk_size: u32::from_le_bytes(array(bytes, 72)?),
            chunk_count: u32::from_le_bytes(array(bytes, 76)?),
            plaintext_length: u64::from_le_bytes(array(bytes, 80)?),
            encoded_length: u64::from_le_bytes(array(bytes, 88)?),
        };
        header.validate(limits)?;
        Ok(header)
    }
}

fn identity(bytes: &[u8], start: usize) -> Result<StableId, StoreError> {
    let id: [u8; 16] = array(bytes, start)?;
    let hex: String = id.iter().map(|b| format!("{b:02x}")).collect();
    StableId::parse(&hex).map_err(|_| StoreError::Identity)
}

fn array<const N: usize>(bytes: &[u8], start: usize) -> Result<[u8; N], StoreError> {
    bytes
        .get(start..start.checked_add(N).ok_or(StoreError::Bounds)?)
        .ok_or(StoreError::Bounds)?
        .try_into()
        .map_err(|_| StoreError::Bounds)
}

#[derive(Debug)]
pub struct Chunk<'a> {
    pub index: u32,
    pub record: &'a [u8; RECORD_SIZE],
    pub ciphertext: &'a [u8],
    pub tag: &'a [u8; TAG_SIZE],
}

/// Parsing checks all chunks and trailing bytes before exposing any record.
pub struct EncryptedObject<'a> {
    pub header: Header,
    header_bytes: &'a [u8; HEADER_SIZE],
    chunks: Vec<Chunk<'a>>,
}

impl<'a> EncryptedObject<'a> {
    pub fn parse(bytes: &'a [u8], limits: Limits) -> Result<Self, StoreError> {
        if bytes.len() as u64 > limits.max_store_bytes {
            return Err(StoreError::Limit);
        }
        let header = Header::parse(bytes, limits)?;
        let overhead = u64::from(header.chunk_count)
            .checked_mul((RECORD_SIZE + TAG_SIZE) as u64)
            .ok_or(StoreError::Bounds)?;
        let expected = (HEADER_SIZE as u64)
            .checked_add(overhead)
            .and_then(|v| v.checked_add(header.encoded_length))
            .ok_or(StoreError::Bounds)?;
        if expected != bytes.len() as u64 {
            return Err(StoreError::Bounds);
        }
        let mut chunks = Vec::with_capacity(header.chunk_count as usize);
        let mut offset = HEADER_SIZE;
        for index in 0..header.chunk_count {
            let record: &[u8; RECORD_SIZE] = bytes
                .get(offset..offset + RECORD_SIZE)
                .ok_or(StoreError::Bounds)?
                .try_into()
                .map_err(|_| StoreError::Bounds)?;
            let length = u32::from_le_bytes(array(record, 4)?);
            let expected_length = (header.encoded_length
                - u64::from(index) * u64::from(header.chunk_size))
            .min(u64::from(header.chunk_size)) as u32;
            if u32::from_le_bytes(array(record, 0)?) != index
                || length != expected_length
                || u32::from_le_bytes(array(record, 8)?) != length
            {
                return Err(StoreError::Chunks);
            }
            offset += RECORD_SIZE;
            let end = offset
                .checked_add(length as usize)
                .ok_or(StoreError::Bounds)?;
            let ciphertext = bytes.get(offset..end).ok_or(StoreError::Bounds)?;
            let tag = bytes
                .get(end..end + TAG_SIZE)
                .ok_or(StoreError::Bounds)?
                .try_into()
                .map_err(|_| StoreError::Bounds)?;
            chunks.push(Chunk {
                index,
                record,
                ciphertext,
                tag,
            });
            offset = end + TAG_SIZE;
        }
        if offset != bytes.len() {
            return Err(StoreError::Bounds);
        }
        Ok(Self {
            header,
            header_bytes: bytes[..HEADER_SIZE]
                .try_into()
                .map_err(|_| StoreError::Bounds)?,
            chunks,
        })
    }

    pub fn chunks(&self) -> &[Chunk<'a>] {
        &self.chunks
    }

    pub fn aad(&self, chunk: &Chunk<'_>) -> [u8; HEADER_SIZE + RECORD_SIZE] {
        aad(self.header_bytes, chunk.record)
    }
}

pub fn aad(
    header: &[u8; HEADER_SIZE],
    record: &[u8; RECORD_SIZE],
) -> [u8; HEADER_SIZE + RECORD_SIZE] {
    let mut bytes = [0; HEADER_SIZE + RECORD_SIZE];
    bytes[..HEADER_SIZE].copy_from_slice(header);
    bytes[HEADER_SIZE..].copy_from_slice(record);
    bytes
}

pub fn nonce(prefix: [u8; 8], index: u32) -> [u8; 12] {
    let mut out = [0; 12];
    out[..8].copy_from_slice(&prefix);
    out[8..].copy_from_slice(&index.to_be_bytes());
    out
}
