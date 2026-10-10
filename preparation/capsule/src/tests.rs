use super::*;
#[path = "../../test_support.rs"]
mod fixtures;

fn capsule() -> Vec<u8> {
    let manifest = fixtures::manifest();
    let id = manifest.private_manifest_id;
    let mut assembler = CapsuleAssembler::new(manifest, Limits::default()).unwrap();
    assembler
        .add_private(
            id,
            ObjectClass::PrivateJson,
            b"{\"message\":\"private synthetic test content\"}",
            CompressionPolicy::Text,
        )
        .unwrap();
    assembler
        .add_public(
            StableId::random(),
            ObjectClass::PublicImage,
            b"public synthetic image bytes",
        )
        .unwrap();
    assembler
        .finish(
            &semver::Version::new(0, 1, 0),
            &[b"private synthetic test content"],
        )
        .unwrap()
}

#[test]
fn complete_capsule_readback_preserves_encrypted_private_object_and_release() {
    let bytes = capsule();
    let loaded =
        LoadedCapsule::parse(&bytes, &semver::Version::new(0, 1, 0), Limits::default()).unwrap();
    assert_eq!(loaded.manifest.objects.len(), 2);
    let engine = QuicknetEngine::new(loaded.manifest.release.clone()).unwrap();
    let beacon = engine.verify_beacon(&fixtures::beacon()).unwrap();
    let key = engine.unlock_cek(loaded.envelope(), &beacon).unwrap();
    let object = loaded.object(loaded.manifest.private_manifest_id).unwrap();
    let plaintext = ag_prepared_object_crypto::decrypt(
        &key,
        Binding {
            project_id: loaded.manifest.project_id,
            build_id: loaded.manifest.build_id,
            object_id: loaded.manifest.private_manifest_id,
        },
        object,
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        &*plaintext,
        b"{\"message\":\"private synthetic test content\"}"
    );
}

#[test]
fn mutations_in_every_section_and_digest_table_fail() {
    let original = capsule();
    let header = CapsuleHeader::parse(&original).unwrap();
    for section in header.sections {
        for offset in [
            section.offset,
            section.offset + section.length / 2,
            section.offset + section.length - 1,
        ] {
            let mut bytes = original.clone();
            bytes[offset as usize] ^= 1;
            assert!(
                LoadedCapsule::parse(&bytes, &semver::Version::new(0, 1, 0), Limits::default())
                    .is_err()
            );
        }
    }
}

#[test]
fn truncation_trailing_bytes_limit_and_version_mismatch_fail() {
    let bytes = capsule();
    for cut in [0, HEADER_SIZE - 1, bytes.len() - 1] {
        assert!(
            LoadedCapsule::parse(
                &bytes[..cut],
                &semver::Version::new(0, 1, 0),
                Limits::default()
            )
            .is_err()
        );
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(
        LoadedCapsule::parse(&trailing, &semver::Version::new(0, 1, 0), Limits::default()).is_err()
    );
    assert!(
        LoadedCapsule::parse(&bytes, &semver::Version::new(0, 0, 0), Limits::default()).is_err()
    );
    assert!(
        LoadedCapsule::parse(
            &bytes,
            &semver::Version::new(0, 1, 0),
            Limits {
                max_store_bytes: 16,
                ..Limits::default()
            }
        )
        .is_err()
    );
}

#[test]
fn empty_public_store_is_canonical_and_missing_private_manifest_is_blocked() {
    let manifest = fixtures::manifest();
    let id = manifest.private_manifest_id;
    let mut assembler = CapsuleAssembler::new(manifest.clone(), Limits::default()).unwrap();
    assembler
        .add_private(id, ObjectClass::PrivateJson, b"{}", CompressionPolicy::Text)
        .unwrap();
    let bytes = assembler
        .finish(&semver::Version::new(0, 1, 0), &[])
        .unwrap();
    LoadedCapsule::parse(&bytes, &semver::Version::new(0, 1, 0), Limits::default()).unwrap();
    let empty = CapsuleAssembler::new(manifest, Limits::default()).unwrap();
    assert!(empty.finish(&semver::Version::new(0, 1, 0), &[]).is_err());
}

#[test]
fn rebuilt_digests_do_not_hide_metadata_binding_or_public_hash_mismatch() {
    let bytes = capsule();
    let loaded =
        LoadedCapsule::parse(&bytes, &semver::Version::new(0, 1, 0), Limits::default()).unwrap();
    let mut manifest = loaded.manifest.clone();
    manifest.objects[0].plaintext_length += 1;
    let rebuilt = serialize(
        &manifest,
        loaded.public_store,
        loaded.private_store,
        loaded.envelope,
        Limits::default(),
    )
    .unwrap();
    assert!(
        LoadedCapsule::parse(&rebuilt, &semver::Version::new(0, 1, 0), Limits::default()).is_err()
    );
    let mut manifest = loaded.manifest.clone();
    manifest.objects[1].protection = ObjectProtection::Public {
        sha256: "00".repeat(32),
    };
    let rebuilt = serialize(
        &manifest,
        loaded.public_store,
        loaded.private_store,
        loaded.envelope,
        Limits::default(),
    )
    .unwrap();
    assert!(
        LoadedCapsule::parse(&rebuilt, &semver::Version::new(0, 1, 0), Limits::default()).is_err()
    );
}

#[test]
fn private_class_cannot_enter_public_store_and_duplicate_ids_fail() {
    let manifest = fixtures::manifest();
    let id = manifest.private_manifest_id;
    let mut assembler = CapsuleAssembler::new(manifest, Limits::default()).unwrap();
    assert!(
        assembler
            .add_public(id, ObjectClass::PrivateJson, b"must not leak")
            .is_err()
    );
    assembler
        .add_private(id, ObjectClass::PrivateJson, b"{}", CompressionPolicy::Text)
        .unwrap();
    assert!(
        assembler
            .add_public(id, ObjectClass::PublicJson, b"{}")
            .is_err()
    );
    assert!(
        assembler
            .add_private(id, ObjectClass::PrivateJson, b"{}", CompressionPolicy::Text)
            .is_err()
    );
}

#[test]
fn plaintext_scan_blocks_actual_private_needles_and_empty_checks() {
    assert!(scan_plaintext(b"runtime text and secret", &[b"secret"]).is_err());
    assert!(scan_plaintext(b"anything", &[b""]).is_err());
    scan_plaintext(
        b"public bytes",
        &[b"private secret longer than public bytes"],
    )
    .unwrap();
}
