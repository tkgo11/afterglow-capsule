#![no_main]
use ag_prepared_object_store::{EncryptedObject, Limits};
use libfuzzer_sys::fuzz_target;

// Unaccepted preparation parser, not a production phase acceptance test.
fuzz_target!(|bytes: &[u8]| {
    if let Ok(object) = EncryptedObject::parse(bytes, Limits::default()) {
        for chunk in object.chunks() {
            let _ = object.aad(chunk);
            let _ = ag_prepared_object_store::nonce(object.header.nonce_prefix, chunk.index);
        }
    }
});
