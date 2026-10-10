#![no_main]
use ag_schema::Manifest;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let viewer_version = semver::Version::new(1, 0, 0);
    if let Ok(manifest) = Manifest::from_json(bytes, &viewer_version) {
        let encoded = manifest.to_json().unwrap();
        assert_eq!(
            manifest,
            Manifest::from_json(&encoded, &viewer_version).unwrap()
        );
    }
});
