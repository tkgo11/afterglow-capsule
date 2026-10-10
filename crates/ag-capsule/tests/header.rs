use ag_capsule::*;

fn header() -> CapsuleHeader {
    CapsuleHeader {
        format_version: 1,
        minimum_reader_version: 1,
        flags: 0,
        sections: [
            Section {
                offset: 128,
                length: 32,
            },
            Section::default(),
            Section {
                offset: 160,
                length: 32,
            },
            Section {
                offset: 192,
                length: 32,
            },
            Section {
                offset: 224,
                length: 32,
            },
        ],
    }
}

fn capsule() -> Vec<u8> {
    let mut bytes = header()
        .encode(256, DEFAULT_MAX_CAPSULE_BYTES)
        .unwrap()
        .to_vec();
    bytes.resize(256, 0);
    bytes
}

#[test]
fn little_endian_header_round_trip_and_section_access() {
    let bytes = capsule();
    assert_eq!(&bytes[..8], b"AGCAPS1\0");
    assert_eq!(&bytes[8..12], &[1, 0, 128, 0]);
    assert_eq!(&bytes[16..24], &128_u64.to_le_bytes());
    assert_eq!(&bytes[96..98], &[1, 0]);
    assert_eq!(CapsuleHeader::parse(&bytes).unwrap(), header());
    for kind in [
        SectionKind::Manifest,
        SectionKind::PrivateStore,
        SectionKind::Timelock,
        SectionKind::DigestTable,
    ] {
        assert_eq!(
            header()
                .section(&bytes, kind, DEFAULT_MAX_CAPSULE_BYTES)
                .unwrap()
                .len(),
            32
        );
    }
    assert!(
        header()
            .section(&bytes, SectionKind::PublicStore, DEFAULT_MAX_CAPSULE_BYTES)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn invalid_magic_versions_header_size_flags_and_reserved_bytes_fail_closed() {
    for offset in [0, 8, 10, 12, 96, 98, 127] {
        let mut bytes = capsule();
        bytes[offset] ^= 0xff;
        assert!(
            CapsuleHeader::parse(&bytes).is_err(),
            "accepted mutation at {offset}"
        );
    }
    for version in [0_u16, 2, u16::MAX] {
        let mut bytes = capsule();
        bytes[8..10].copy_from_slice(&version.to_le_bytes());
        assert_eq!(CapsuleHeader::parse(&bytes), Err(CapsuleError::Version));
    }
    let mut bytes = capsule();
    bytes[96..98].copy_from_slice(&0_u16.to_le_bytes());
    assert_eq!(CapsuleHeader::parse(&bytes), Err(CapsuleError::Version));
}

#[test]
fn truncation_overflow_overlap_and_missing_sections_are_rejected() {
    let bytes = capsule();
    for length in 0..bytes.len() {
        assert!(
            CapsuleHeader::parse(&bytes[..length]).is_err(),
            "accepted truncation at {length}"
        );
    }
    for offset in [0, 127, 255, 256, u64::MAX] {
        let mut bytes = capsule();
        bytes[16..24].copy_from_slice(&offset.to_le_bytes());
        assert!(CapsuleHeader::parse(&bytes).is_err());
    }
    for length in [0, 129, u64::MAX] {
        let mut bytes = capsule();
        bytes[24..32].copy_from_slice(&length.to_le_bytes());
        assert!(CapsuleHeader::parse(&bytes).is_err());
    }
    let mut bytes = capsule();
    bytes[48..56].copy_from_slice(&159_u64.to_le_bytes());
    assert_eq!(CapsuleHeader::parse(&bytes), Err(CapsuleError::Overlap));
    let mut bytes = capsule();
    bytes[32..40].copy_from_slice(&128_u64.to_le_bytes());
    assert_eq!(CapsuleHeader::parse(&bytes), Err(CapsuleError::Bounds));
}

#[test]
fn size_override_changes_only_size_policy() {
    let bytes = capsule();
    assert_eq!(
        CapsuleHeader::parse_with_limit(&bytes, 255),
        Err(CapsuleError::TooLarge)
    );
    assert_eq!(
        CapsuleHeader::parse_with_limit(&bytes, 256).unwrap(),
        header()
    );
    let mut invalid = bytes;
    invalid[12] = 1;
    assert!(CapsuleHeader::parse_with_limit(&invalid, u64::MAX).is_err());
}

#[test]
fn section_access_rechecks_extent_even_for_programmatically_constructed_headers() {
    let mut header = header();
    header.sections[0].offset = u64::MAX;
    assert!(
        header
            .section(&capsule(), SectionKind::Manifest, DEFAULT_MAX_CAPSULE_BYTES)
            .is_err()
    );
    assert!(header.encode(256, DEFAULT_MAX_CAPSULE_BYTES).is_err());
}
