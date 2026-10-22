#![no_main]
use ag_capsule::{CapsuleHeader, DEFAULT_MAX_CAPSULE_BYTES, SectionKind};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(header) = CapsuleHeader::parse(bytes) {
        for kind in [
            SectionKind::Manifest,
            SectionKind::PublicStore,
            SectionKind::PrivateStore,
            SectionKind::Timelock,
            SectionKind::DigestTable,
        ] {
            assert!(
                header
                    .section(bytes, kind, DEFAULT_MAX_CAPSULE_BYTES)
                    .is_ok()
            );
        }
        let encoded = header
            .encode(bytes.len() as u64, DEFAULT_MAX_CAPSULE_BYTES)
            .unwrap();
        assert_eq!(&bytes[..encoded.len()], &encoded);
    }
});
