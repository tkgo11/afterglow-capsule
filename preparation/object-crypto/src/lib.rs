//! Standard primitives for isolated Phase 3 preparation. No release logic.
//! Neither production application depends on this crate while Phase 2 is pending.

use std::{
    collections::HashSet,
    io::{Cursor, Read},
};

use aes_gcm::{Aes256Gcm, KeyInit, Nonce, Tag, aead::AeadInPlace};
use ag_prepared_object_store::{
    Binding, Compression, DEFAULT_CHUNK_SIZE, EncryptedObject, HEADER_SIZE, Header, Limits,
    RECORD_SIZE, StoreError, aad, nonce,
};
use ag_schema::StableId;
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("operating-system cryptographic random generator failed")]
    Random,
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("object key derivation failed")]
    Derivation,
    #[error("object authentication or identity binding failed")]
    Authentication,
    #[error("object identity was already encrypted in this build")]
    DuplicateObject,
    #[error("unable to generate a unique nonce prefix")]
    NonceExhaustion,
    #[error("object compression/decompression failed or exceeded limits")]
    Compression,
}

/// Non-cloneable secret, redacted Debug, zeroized on drop. Never serialize a CEK.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ContentKey([u8; 32]);

impl std::fmt::Debug for ContentKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ContentKey([REDACTED])")
    }
}

impl ContentKey {
    pub fn generate() -> Result<Self, CryptoError> {
        let mut key = Self([0; 32]);
        getrandom::fill(&mut key.0).map_err(|_| CryptoError::Random)?;
        Ok(key)
    }

    /// Only the timelock adapter will call this with an authenticated CEK.
    /// This primitive cannot authorize release; application integration is gated.
    pub fn from_unwrapped(bytes: Zeroizing<[u8; 32]>) -> Self {
        Self(*bytes)
    }

    pub fn with_secret<T>(&self, operation: impl FnOnce(&[u8; 32]) -> T) -> T {
        operation(&self.0)
    }
}

fn object_key(cek: &ContentKey, binding: Binding) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let kdf = Hkdf::<Sha256>::new(Some(binding.project_id.as_bytes()), &cek.0);
    let mut info = b"AFTERGLOW/object/v1/".to_vec();
    info.extend_from_slice(binding.object_id.as_bytes());
    let mut key = Zeroizing::new([0; 32]);
    kdf.expand(&info, &mut *key)
        .map_err(|_| CryptoError::Derivation)?;
    Ok(key)
}

#[derive(Clone, Copy, Debug)]
pub enum CompressionPolicy {
    None,
    /// Measure the Zstandard result, retaining it only when smaller.
    Text,
}

/// One encryptor per frozen build. Never re-encrypt an ID or reuse a prefix.
pub struct BuildEncryptor {
    key: ContentKey,
    project_id: StableId,
    build_id: StableId,
    used_ids: HashSet<StableId>,
    prefixes: HashSet<[u8; 8]>,
    limits: Limits,
}

impl BuildEncryptor {
    pub fn new(
        project_id: StableId,
        build_id: StableId,
        limits: Limits,
    ) -> Result<Self, CryptoError> {
        Ok(Self {
            key: ContentKey::generate()?,
            project_id,
            build_id,
            used_ids: HashSet::new(),
            prefixes: HashSet::new(),
            limits,
        })
    }

    pub fn with_cek<T>(&self, operation: impl FnOnce(&ContentKey) -> T) -> T {
        operation(&self.key)
    }

    pub fn encrypt(
        &mut self,
        object_id: StableId,
        plaintext: &[u8],
        policy: CompressionPolicy,
    ) -> Result<Vec<u8>, CryptoError> {
        if !self.used_ids.insert(object_id) {
            return Err(CryptoError::DuplicateObject);
        }
        let mut prefix = [0; 8];
        let mut selected = false;
        for _ in 0..32 {
            getrandom::fill(&mut prefix).map_err(|_| CryptoError::Random)?;
            if self.prefixes.insert(prefix) {
                selected = true;
                break;
            }
        }
        if !selected {
            return Err(CryptoError::NonceExhaustion);
        }
        encrypt(
            &self.key,
            Binding {
                project_id: self.project_id,
                build_id: self.build_id,
                object_id,
            },
            prefix,
            plaintext,
            policy,
            DEFAULT_CHUNK_SIZE,
            self.limits,
        )
    }
}

fn encrypt(
    cek: &ContentKey,
    binding: Binding,
    nonce_prefix: [u8; 8],
    plaintext: &[u8],
    policy: CompressionPolicy,
    chunk_size: u32,
    limits: Limits,
) -> Result<Vec<u8>, CryptoError> {
    if plaintext.len() as u64 > limits.max_plaintext_bytes {
        return Err(StoreError::Limit.into());
    }
    let compressed = if matches!(policy, CompressionPolicy::Text) && !plaintext.is_empty() {
        let mut compressor =
            zstd::bulk::Compressor::new(3).map_err(|_| CryptoError::Compression)?;
        compressor
            .set_parameter(zstd::zstd_safe::CParameter::WindowLog(23))
            .map_err(|_| CryptoError::Compression)?;
        Some(Zeroizing::new(
            compressor
                .compress(plaintext)
                .map_err(|_| CryptoError::Compression)?,
        ))
    } else {
        None
    };
    let (encoded, compression) = match compressed.as_ref() {
        Some(bytes) if bytes.len() < plaintext.len() => (bytes.as_slice(), Compression::Zstandard),
        _ => (plaintext, Compression::None),
    };
    if chunk_size == 0 {
        return Err(StoreError::Chunks.into());
    }
    let count = (encoded.len() as u64)
        .div_ceil(u64::from(chunk_size))
        .max(1);
    let header = Header {
        binding,
        compression,
        nonce_prefix,
        chunk_size,
        chunk_count: u32::try_from(count).map_err(|_| StoreError::Chunks)?,
        plaintext_length: plaintext.len() as u64,
        encoded_length: encoded.len() as u64,
    };
    let header_bytes = header.encode(limits)?;
    let total = (HEADER_SIZE as u64) + header.encoded_length + count * (RECORD_SIZE as u64 + 16);
    if total > limits.max_store_bytes {
        return Err(StoreError::Limit.into());
    }
    let key = object_key(cek, binding)?;
    let cipher = Aes256Gcm::new_from_slice(&*key).map_err(|_| CryptoError::Derivation)?;
    let mut output = Vec::with_capacity(usize::try_from(total).map_err(|_| StoreError::Limit)?);
    output.extend_from_slice(&header_bytes);
    for index in 0..header.chunk_count {
        let start = (u64::from(index) * u64::from(chunk_size)) as usize;
        let end = encoded.len().min(start + chunk_size as usize);
        let mut chunk = Zeroizing::new(encoded[start..end].to_vec());
        let mut record = [0; RECORD_SIZE];
        record[..4].copy_from_slice(&index.to_le_bytes());
        record[4..8].copy_from_slice(&(chunk.len() as u32).to_le_bytes());
        record[8..12].copy_from_slice(&(chunk.len() as u32).to_le_bytes());
        let tag = cipher
            .encrypt_in_place_detached(
                Nonce::from_slice(&nonce(nonce_prefix, index)),
                &aad(&header_bytes, &record),
                &mut chunk,
            )
            .map_err(|_| CryptoError::Authentication)?;
        output.extend_from_slice(&record);
        output.extend_from_slice(&chunk);
        output.extend_from_slice(&tag);
    }
    Ok(output)
}

/// On-demand, in-memory object decryption. No bytes escape until ALL tags pass.
pub fn decrypt(
    cek: &ContentKey,
    expected: Binding,
    bytes: &[u8],
    limits: Limits,
) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    let object = EncryptedObject::parse(bytes, limits)?;
    if object.header.binding != expected {
        return Err(CryptoError::Authentication);
    }
    let key = object_key(cek, expected)?;
    let cipher = Aes256Gcm::new_from_slice(&*key).map_err(|_| CryptoError::Derivation)?;
    let mut encoded = Zeroizing::new(Vec::with_capacity(object.header.encoded_length as usize));
    for chunk in object.chunks() {
        let mut plaintext = Zeroizing::new(chunk.ciphertext.to_vec());
        cipher
            .decrypt_in_place_detached(
                Nonce::from_slice(&nonce(object.header.nonce_prefix, chunk.index)),
                &object.aad(chunk),
                &mut plaintext,
                Tag::from_slice(chunk.tag),
            )
            .map_err(|_| CryptoError::Authentication)?;
        encoded.extend_from_slice(&plaintext);
    }
    match object.header.compression {
        Compression::None => Ok(encoded),
        Compression::Zstandard => {
            let mut decoder =
                zstd::stream::read::Decoder::with_buffer(Cursor::new(encoded.as_slice()))
                    .map_err(|_| CryptoError::Compression)?
                    .single_frame();
            decoder
                .window_log_max(23)
                .map_err(|_| CryptoError::Compression)?;
            let mut output = Zeroizing::new(Vec::new());
            let mut buffer = Zeroizing::new([0; 16 * 1024]);
            loop {
                let read = decoder
                    .read(&mut *buffer)
                    .map_err(|_| CryptoError::Compression)?;
                if read == 0 {
                    break;
                }
                if (output.len() as u64)
                    .checked_add(read as u64)
                    .is_none_or(|n| n > object.header.plaintext_length)
                {
                    return Err(CryptoError::Compression);
                }
                output.extend_from_slice(&buffer[..read]);
            }
            if output.len() as u64 != object.header.plaintext_length
                || decoder.finish().position() != encoded.len() as u64
            {
                return Err(CryptoError::Compression);
            }
            Ok(output)
        }
    }
}

#[cfg(test)]
mod tests;
