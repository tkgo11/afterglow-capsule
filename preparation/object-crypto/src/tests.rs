use super::*;
use ag_prepared_object_store::{MAX_CHUNK_SIZE, TAG_SIZE};

fn binding() -> Binding {
    Binding {
        project_id: StableId::parse("00112233-4455-4677-8899-aabbccddeeff").unwrap(),
        build_id: StableId::parse("10213243-5465-4768-899a-abbccddeeff0").unwrap(),
        object_id: StableId::parse("21324354-6576-4879-9aab-bccddeeff001").unwrap(),
    }
}

fn key() -> ContentKey {
    ContentKey(std::array::from_fn(|n| n as u8))
}

fn sealed(plaintext: &[u8], policy: CompressionPolicy, chunk: u32) -> Vec<u8> {
    encrypt(
        &key(),
        binding(),
        [7; 8],
        plaintext,
        policy,
        chunk,
        Limits::default(),
    )
    .unwrap()
}

fn open(bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    decrypt(&key(), binding(), bytes, Limits::default())
}

#[test]
fn independent_hkdf_and_chunked_gcm_vector() {
    let vector: serde_json::Value =
        serde_json::from_str(include_str!("../tests/public-vector.json")).unwrap();
    let expected = hex::decode(vector["object_key"].as_str().unwrap()).unwrap();
    assert_eq!(
        &*object_key(&key(), binding()).unwrap(),
        expected.as_slice()
    );
    let plaintext = hex::decode(vector["plaintext"].as_str().unwrap()).unwrap();
    let prefix = hex::decode(vector["nonce_prefix"].as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let actual = encrypt(
        &key(),
        binding(),
        prefix,
        &plaintext,
        CompressionPolicy::None,
        vector["chunk_size"].as_u64().unwrap() as u32,
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        hex::encode(&actual),
        vector["encrypted_object"].as_str().unwrap()
    );
    assert_eq!(*open(&actual).unwrap(), plaintext);
}

#[test]
fn cek_uses_os_randomness_and_debug_is_redacted() {
    let first = ContentKey::generate().unwrap();
    let second = ContentKey::generate().unwrap();
    first.with_secret(|a| {
        second.with_secret(|b| {
            assert_eq!(a.len(), 32);
            assert_ne!(a, b);
        })
    });
    assert_eq!(format!("{first:?}"), "ContentKey([REDACTED])");
}

#[test]
fn build_enforces_unique_ids_and_prefixes() {
    let mut builder =
        BuildEncryptor::new(binding().project_id, binding().build_id, Limits::default()).unwrap();
    let mut nonces = HashSet::new();
    for _ in 0..64 {
        let id = StableId::random();
        let encrypted = builder
            .encrypt(id, b"private", CompressionPolicy::None)
            .unwrap();
        let object = EncryptedObject::parse(&encrypted, Limits::default()).unwrap();
        assert_eq!(object.header.chunk_size, DEFAULT_CHUNK_SIZE);
        assert!(nonces.insert(object.header.nonce_prefix));
        assert!(matches!(
            builder.encrypt(id, b"again", CompressionPolicy::None),
            Err(CryptoError::DuplicateObject)
        ));
    }
}

#[test]
fn empty_small_and_multi_megabyte_objects_round_trip() {
    for plaintext in [
        vec![],
        b"small".to_vec(),
        vec![0x5a; 2 * DEFAULT_CHUNK_SIZE as usize + 7],
    ] {
        let bytes = sealed(&plaintext, CompressionPolicy::None, DEFAULT_CHUNK_SIZE);
        assert_eq!(*open(&bytes).unwrap(), plaintext);
    }
}

#[test]
fn compression_is_measured_and_compressed_before_encryption() {
    let bytes = sealed(
        &vec![b'a'; 2_000_000],
        CompressionPolicy::Text,
        DEFAULT_CHUNK_SIZE,
    );
    let object = EncryptedObject::parse(&bytes, Limits::default()).unwrap();
    assert_eq!(object.header.compression, Compression::Zstandard);
    assert!(object.header.encoded_length < object.header.plaintext_length);
    assert_eq!(open(&bytes).unwrap().len(), 2_000_000);
    let tiny = sealed(b"x", CompressionPolicy::Text, 16);
    assert_eq!(
        Header::parse(&tiny, Limits::default()).unwrap().compression,
        Compression::None
    );
}

#[test]
fn every_ciphertext_tag_and_header_byte_modification_is_rejected() {
    let original = sealed(
        b"authenticated multi chunk private content",
        CompressionPolicy::None,
        8,
    );
    for offset in 0..original.len() {
        let mut bytes = original.clone();
        bytes[offset] ^= 1;
        assert!(open(&bytes).is_err(), "mutation accepted at {offset}");
    }
}

#[test]
fn every_truncation_and_trailing_data_is_rejected() {
    let original = sealed(b"truncation at every byte", CompressionPolicy::None, 8);
    for length in 0..original.len() {
        assert!(open(&original[..length]).is_err());
    }
    let mut appended = original.clone();
    appended.push(0);
    assert!(open(&appended).is_err());
}

#[test]
fn wrong_cek_project_build_object_and_record_order_are_rejected() {
    let original = sealed(&[1; 32], CompressionPolicy::None, 8);
    assert!(
        decrypt(
            &ContentKey([0; 32]),
            binding(),
            &original,
            Limits::default()
        )
        .is_err()
    );
    for wrong in [
        Binding {
            project_id: StableId::random(),
            ..binding()
        },
        Binding {
            build_id: StableId::random(),
            ..binding()
        },
        Binding {
            object_id: StableId::random(),
            ..binding()
        },
    ] {
        assert!(decrypt(&key(), wrong, &original, Limits::default()).is_err());
    }
    let mut swapped = original;
    let size = RECORD_SIZE + 8 + TAG_SIZE;
    let first = swapped[HEADER_SIZE..HEADER_SIZE + size].to_vec();
    let second = swapped[HEADER_SIZE + size..HEADER_SIZE + 2 * size].to_vec();
    swapped[HEADER_SIZE..HEADER_SIZE + size].copy_from_slice(&second);
    swapped[HEADER_SIZE + size..HEADER_SIZE + 2 * size].copy_from_slice(&first);
    assert!(open(&swapped).is_err());
}

#[test]
fn removing_last_chunk_even_with_adjusted_lengths_cannot_authenticate() {
    let mut bytes = sealed(&[3; 32], CompressionPolicy::None, 8);
    bytes.truncate(bytes.len() - RECORD_SIZE - 8 - TAG_SIZE);
    bytes[76..80].copy_from_slice(&3_u32.to_le_bytes());
    bytes[80..88].copy_from_slice(&24_u64.to_le_bytes());
    bytes[88..96].copy_from_slice(&24_u64.to_le_bytes());
    assert!(matches!(open(&bytes), Err(CryptoError::Authentication)));
}

#[test]
fn nonce_is_prefix_and_big_endian_counter() {
    assert_eq!(
        nonce([1; 8], 0x01020304),
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 3, 4]
    );
    assert_ne!(nonce([1; 8], 0), nonce([1; 8], u32::MAX));
}

#[test]
fn configured_limits_and_chunk_overflow_fail_before_allocation() {
    let bytes = sealed(b"bounded", CompressionPolicy::None, 8);
    for limits in [
        Limits {
            max_plaintext_bytes: 1,
            ..Limits::default()
        },
        Limits {
            max_store_bytes: 1,
            ..Limits::default()
        },
        Limits {
            max_chunks: 0,
            ..Limits::default()
        },
    ] {
        assert!(decrypt(&key(), binding(), &bytes, limits).is_err());
    }
    let mut header = Header::parse(&bytes, Limits::default()).unwrap();
    header.chunk_size = 1;
    header.plaintext_length = u64::from(u32::MAX) + 1;
    header.encoded_length = header.plaintext_length;
    header.chunk_count = u32::MAX;
    assert!(
        header
            .validate(Limits {
                max_plaintext_bytes: u64::MAX,
                max_chunks: u32::MAX,
                max_store_bytes: u64::MAX
            })
            .is_err()
    );
    header.chunk_size = MAX_CHUNK_SIZE + 1;
    assert!(header.validate(Limits::default()).is_err());
}

#[test]
fn malformed_compressed_frame_is_rejected_after_authentication() {
    // Re-authenticate deliberately malformed public fixture compressed bytes.
    // This test-only construction is absent from non-test library APIs.
    let original = vec![b'z'; 2048];
    let mut bytes = sealed(&original, CompressionPolicy::Text, 1024);
    let object = EncryptedObject::parse(&bytes, Limits::default()).unwrap();
    let header_bytes: [u8; HEADER_SIZE] = bytes[..HEADER_SIZE].try_into().unwrap();
    let record = *object.chunks()[0].record;
    let len = object.chunks()[0].ciphertext.len();
    let mut bad = vec![0; len];
    let derived = object_key(&key(), binding()).unwrap();
    let cipher = Aes256Gcm::new_from_slice(&*derived).unwrap();
    let tag = cipher
        .encrypt_in_place_detached(
            Nonce::from_slice(&nonce([7; 8], 0)),
            &aad(&header_bytes, &record),
            &mut bad,
        )
        .unwrap();
    bytes[HEADER_SIZE + RECORD_SIZE..HEADER_SIZE + RECORD_SIZE + len].copy_from_slice(&bad);
    bytes[HEADER_SIZE + RECORD_SIZE + len..].copy_from_slice(&tag);
    assert!(matches!(open(&bytes), Err(CryptoError::Compression)));
}
