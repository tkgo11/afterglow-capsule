//! Public test fixtures, referenced only by test targets.
use ag_schema::{Manifest, PinnedRelease, StableId};

pub fn fixture() -> serde_json::Value {
    let text = include_str!("../docs/external-assumptions.md");
    let json = text
        .split("<!-- BEGIN AFTERGLOW HISTORICAL QUICKNET FIXTURE -->")
        .nth(1)
        .unwrap()
        .split("```json")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    serde_json::from_str(json).unwrap()
}

pub fn release(round: u64) -> PinnedRelease {
    let chain = fixture()["chain"].clone();
    let genesis = chain["genesis_time"].as_u64().unwrap();
    let period = chain["period"].as_u64().unwrap();
    let time = |n| {
        chrono::DateTime::from_timestamp(i64::try_from(n).unwrap(), 0)
            .unwrap()
            .to_rfc3339()
    };
    serde_json::from_value(serde_json::json!({
        "utc": time(genesis + (round - 1) * period), "timezone": "UTC",
        "target_round": round, "timelock_profile": "quicknet-v1",
        "network": { "profile_id": "quicknet-v1", "chain_hash": chain["hash"],
            "public_key": chain["public_key"], "genesis_utc": time(genesis),
            "period_seconds": period, "scheme_id": chain["schemeID"],
            "relays": ["https://relay-one.example", "https://relay-two.example"] }
    }))
    .unwrap()
}

pub fn manifest() -> Manifest {
    serde_json::from_value(serde_json::json!({
        "format_name": "afterglow-project", "format_version": 1, "minimum_reader_version": 1,
        "project_id": StableId::random(), "build_id": StableId::random(),
        "viewer_min_version": "0.1.0", "identity": { "title": "Public synthetic archive" },
        "release": release(1000), "layout": { "pre_release": "countdown-default", "post_release": "portrait-grid" },
        "theme_id": "temporal-glass-dark", "private_manifest_id": StableId::random(), "objects": []
    })).unwrap()
}

pub fn beacon() -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let value = fixture()["additional_quicknet_beacon"].clone();
    let signature = value["signature"].as_str().unwrap();
    serde_json::to_vec(
        &serde_json::json!({ "round": value["round"], "signature": signature,
        "randomness": hex::encode(Sha256::digest(hex::decode(signature).unwrap())) }),
    )
    .unwrap()
}
