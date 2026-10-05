#![no_main]
use ag_schema::{ObjectMetadata, bounded_json, validate_object_table};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(objects) = bounded_json::<Vec<ObjectMetadata>>(bytes) {
        let _ = validate_object_table(&objects);
    }
});
