use ag_schema::*;
use serde_json::{Value, json};

fn read_manifest(value: &Value) -> Result<Manifest, SchemaError> {
    Manifest::from_json(
        &serde_json::to_vec(value).unwrap(),
        &semver::Version::new(1, 0, 0),
    )
}

fn manifest_value() -> Value {
    // Synthetic network values deliberately cannot serve as production profiles.
    json!({
        "format_name": "afterglow-project", "format_version": 1,
        "minimum_reader_version": 1,
        "project_id": "2ee81f75-44be-4d52-82c1-1c6c104dbd52",
        "build_id": "c2c4774f-1ad8-45cc-ac90-e867de6ddbf1",
        "viewer_min_version": "0.1.0",
        "identity": {"title": "Generic archive", "subtitle": "Future"},
        "release": {
            "utc": "2030-03-01T15:00:01Z", "timezone": "Asia/Seoul",
            "target_round": 2, "timelock_profile": "synthetic-only",
            "network": {
                "profile_id": "synthetic-only", "chain_hash": "00".repeat(32),
                "public_key": "01".repeat(48), "genesis_utc": "2030-03-01T15:00:00Z",
                "period_seconds": 3, "scheme_id": "synthetic-only",
                "relays": ["https://relay-one.example", "https://relay-two.example"]
            }
        },
        "layout": {"pre_release": "countdown-default", "post_release": "constellation"},
        "theme_id": "temporal-glass-dark",
        "private_manifest_id": "58df0307-79a5-4eb8-bd67-46c71a3bd979",
        "objects": [{
            "object_id": "58df0307-79a5-4eb8-bd67-46c71a3bd979",
            "class": "private-json", "offset": 0, "length": 64,
            "plaintext_length": 4, "compression": "none",
            "protection": {"kind": "private", "nonce_prefix": [1,2,3,4,5,6,7,8], "chunk_count": 1, "chunk_size": 1048576}
        }]
    })
}

#[test]
fn public_manifest_round_trips_with_pinned_metadata() {
    let manifest = read_manifest(&manifest_value()).unwrap();
    let encoded = manifest.to_json().unwrap();
    assert_eq!(
        manifest,
        Manifest::from_json(&encoded, &semver::Version::new(1, 0, 0)).unwrap()
    );
    assert!(!String::from_utf8(encoded).unwrap().contains("filename"));
}

#[test]
fn versions_and_required_fields_fail_closed() {
    for (field, invalid) in [
        ("format_name", json!("other-format")),
        ("format_version", json!(0)),
        ("format_version", json!(2)),
        ("minimum_reader_version", json!(0)),
        ("minimum_reader_version", json!(2)),
        ("viewer_min_version", json!("9.0.0")),
        ("project_id", json!("00000000-0000-0000-0000-000000000000")),
    ] {
        let mut value = manifest_value();
        value[field] = invalid;
        assert!(read_manifest(&value).is_err(), "accepted {field}");
    }
    let mut value = manifest_value();
    value
        .as_object_mut()
        .unwrap()
        .remove("minimum_reader_version");
    assert!(read_manifest(&value).is_err());
}

#[test]
fn unknown_optional_fields_are_ignored_but_duplicate_critical_fields_are_rejected() {
    let mut value = manifest_value();
    value["optional_future_description"] = json!("ignored");
    assert!(read_manifest(&value).is_ok());
    let json = serde_json::to_string(&value).unwrap();
    let duplicate = json.replacen("{", "{\"format_version\":2,", 1);
    assert!(Manifest::from_json(duplicate.as_bytes(), &semver::Version::new(1, 0, 0)).is_err());
}

#[test]
fn pinned_release_rejects_zero_wrong_round_wrong_profile_timezone_and_insecure_relays() {
    for round in [0, 1, 3, u64::MAX] {
        let mut value = manifest_value();
        value["release"]["target_round"] = json!(round);
        assert!(read_manifest(&value).is_err());
    }
    for (field, invalid) in [("timezone", "not/a-zone"), ("timelock_profile", "other")] {
        let mut value = manifest_value();
        value["release"][field] = json!(invalid);
        assert!(read_manifest(&value).is_err());
    }
    for relay in [
        "http://relay.example",
        "https://user:password@relay.example",
        "https://relay.example/?project=unique",
        "https://relay.example/#fragment",
    ] {
        let mut value = manifest_value();
        value["release"]["network"]["relays"][0] = json!(relay);
        assert!(read_manifest(&value).is_err());
    }
    let mut value = manifest_value();
    value["release"]["network"]["period_seconds"] = json!(0);
    assert!(read_manifest(&value).is_err());
}

#[test]
fn round_ceiling_handles_subsecond_boundaries_and_long_horizons() {
    let genesis = "2030-03-01T15:00:00Z".parse().unwrap();
    for (instant, expected) in [
        ("2030-03-01T15:00:00Z", 1),
        ("2030-03-01T15:00:00.000000001Z", 2),
        ("2030-03-01T15:00:03Z", 2),
        ("2030-03-01T15:00:03.000000001Z", 3),
    ] {
        assert_eq!(
            target_round(instant.parse().unwrap(), genesis, 3).unwrap(),
            expected
        );
    }
    assert!(target_round("2030-03-01T14:59:59Z".parse().unwrap(), genesis, 3).is_err());
    assert!(target_round(genesis, genesis, 0).is_err());
    assert!(target_round("2500-01-01T00:00:00Z".parse().unwrap(), genesis, 3).unwrap() > 1);
}

#[test]
fn object_table_rejects_private_plaintext_metadata_duplicate_ids_nonce_reuse_and_overflow() {
    let manifest = read_manifest(&manifest_value()).unwrap();
    let private = manifest.objects[0].clone();
    let mut invalid = private.clone();
    invalid.protection = ObjectProtection::Public {
        sha256: "ab".repeat(32),
    };
    assert!(invalid.validate().is_err());
    assert!(validate_object_table(&[private.clone(), private.clone()]).is_err());
    invalid = private.clone();
    invalid.object_id = StableId::random();
    invalid.offset = 64;
    assert!(validate_object_table(&[private.clone(), invalid]).is_err());
    invalid = private.clone();
    invalid.offset = u64::MAX;
    assert!(invalid.validate().is_err());
    invalid = private.clone();
    invalid.length = 0;
    assert!(invalid.validate().is_err());
    invalid = private.clone();
    invalid.protection = ObjectProtection::Private {
        nonce_prefix: [0; 8],
        chunk_count: 0,
        chunk_size: 1,
    };
    assert!(invalid.validate().is_err());
    let mut second = private.clone();
    second.object_id = StableId::random();
    second.offset = 63;
    second.protection = ObjectProtection::Private {
        nonce_prefix: [9; 8],
        chunk_count: 1,
        chunk_size: 1,
    };
    assert!(validate_object_table(&[private, second]).is_err());
    let mut value = manifest_value();
    value["objects"] = json!([]);
    assert!(read_manifest(&value).is_err());
}

#[test]
fn public_objects_require_valid_sha256_and_separate_store_ranges() {
    let manifest = read_manifest(&manifest_value()).unwrap();
    let mut object = manifest.objects[0].clone();
    object.object_id = StableId::random();
    object.class = ObjectClass::PublicImage;
    object.protection = ObjectProtection::Public {
        sha256: "ab".repeat(32),
    };
    validate_object_table(&[manifest.objects[0].clone(), object.clone()]).unwrap();
    validate_object_store_bounds(&[manifest.objects[0].clone(), object.clone()], 64, 64).unwrap();
    assert!(validate_object_store_bounds(&[object.clone()], 63, 64).is_err());
    assert!(validate_object_store_bounds(&manifest.objects, 64, 63).is_err());
    for digest in ["".to_owned(), "g".repeat(64), "a".repeat(63)] {
        object.protection = ObjectProtection::Public { sha256: digest };
        assert!(object.validate().is_err());
    }
}

#[test]
fn identifier_generation_and_wire_versions_are_validated() {
    let id = StableId::random();
    assert_eq!(StableId::parse(&id.to_string()).unwrap(), id);
    assert_ne!(StableId::random(), id);
    assert!(StableId::parse("a77850ef-d2ef-5bb2-9d4a-e383877e816c").is_err());
    assert!(StableId::parse("01890f3e-c7b5-7cc5-8bc6-9a1f7bccbf18").is_ok());
    let document = Document::new("afterglow-terminology", Terminology::default()).unwrap();
    let bytes = document.to_json("afterglow-terminology").unwrap();
    assert_eq!(
        document,
        Document::<Terminology>::from_json(&bytes, "afterglow-terminology").unwrap()
    );
    assert!(Document::<Terminology>::from_json(&bytes, "afterglow-project").is_err());
    let mut invalid = document;
    invalid.minimum_reader_version = 2;
    assert!(invalid.to_json("afterglow-terminology").is_err());
}

#[test]
fn json_limit_and_recursion_limit_are_retained() {
    let oversized = vec![b' '; MAX_JSON_BYTES + 1];
    assert!(matches!(
        bounded_json::<Value>(&oversized),
        Err(SchemaError::TooLarge(_))
    ));
    let nested = format!("{}0{}", "[".repeat(200), "]".repeat(200));
    assert!(bounded_json::<Value>(nested.as_bytes()).is_err());
    for bytes in [b"null".as_slice(), b"{", b"\xff", b"[]"] {
        assert!(Manifest::from_json(bytes, &semver::Version::new(1, 0, 0)).is_err());
    }
}
