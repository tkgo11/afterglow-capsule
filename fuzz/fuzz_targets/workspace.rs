#![no_main]
use ag_project::{ProjectDocument, WORKSPACE_FORMAT};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(project) = ProjectDocument::from_json(bytes, WORKSPACE_FORMAT) {
        let encoded = project.to_json(WORKSPACE_FORMAT).unwrap();
        assert_eq!(
            project,
            ProjectDocument::from_json(&encoded, WORKSPACE_FORMAT).unwrap()
        );
    }
});
