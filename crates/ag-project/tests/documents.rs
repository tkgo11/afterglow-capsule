use ag_project::*;
use ag_schema::*;

fn project() -> ProjectDocument {
    let project_id = StableId::random();
    let contributor_id = StableId::random();
    Document::new(
        WORKSPACE_FORMAT,
        Project {
            project_id,
            identity: Identity {
                title: "Generic archive".into(),
                subtitle: String::new(),
                introduction: String::new(),
                creator_credit: String::new(),
            },
            locale: "en-US".into(),
            terminology: Document::new(TERMINOLOGY_FORMAT, Terminology::default()).unwrap(),
            schema: Document::new(SCHEMA_FORMAT, CustomSchema::default()).unwrap(),
            release: Document::new(
                RELEASE_FORMAT,
                RequestedRelease {
                    utc: "2030-01-01T00:00:00Z".parse().unwrap(),
                    timezone: "Etc/UTC".into(),
                    timelock_profile: "unselected".into(),
                },
            )
            .unwrap(),
            contributors: vec![
                Document::new(
                    CONTRIBUTOR_FORMAT,
                    Contributor {
                        project_id,
                        contributor_id,
                        status: ContributorStatus::Draft,
                        fields: FieldValues::new(),
                    },
                )
                .unwrap(),
            ],
            entries: vec![
                Document::new(
                    ENTRY_FORMAT,
                    Entry {
                        project_id,
                        entry_id: StableId::random(),
                        contributor_id,
                        title: "Draft".into(),
                        blocks: vec![ContentBlock::Divider],
                    },
                )
                .unwrap(),
            ],
        },
    )
    .unwrap()
}

#[test]
fn generic_editable_project_round_trips_without_a_build_or_release_key() {
    let project = project();
    let bytes = project.to_json(WORKSPACE_FORMAT).unwrap();
    assert_eq!(
        project,
        ProjectDocument::from_json(&bytes, WORKSPACE_FORMAT).unwrap()
    );
    assert!(!String::from_utf8(bytes).unwrap().contains("target_round"));
}

#[test]
fn wrong_project_duplicate_ids_or_missing_contributor_are_rejected() {
    let mut project = project();
    project.data.contributors[0].data.project_id = StableId::random();
    assert!(project.validate_as(WORKSPACE_FORMAT).is_err());
    let mut project = self::project();
    project.data.entries[0].data.project_id = StableId::random();
    assert!(project.validate_as(WORKSPACE_FORMAT).is_err());
    let mut project = self::project();
    project.data.entries[0].data.contributor_id = StableId::random();
    assert!(project.validate_as(WORKSPACE_FORMAT).is_err());
    let mut project = self::project();
    project
        .data
        .contributors
        .push(project.data.contributors[0].clone());
    assert!(project.validate_as(WORKSPACE_FORMAT).is_err());
    let mut project = self::project();
    project.data.entries.push(project.data.entries[0].clone());
    assert!(project.validate_as(WORKSPACE_FORMAT).is_err());
}

#[test]
fn nested_document_versions_and_workspace_manifest_mixups_are_rejected() {
    let mut project = project();
    project.data.release.format_version = 2;
    assert!(project.to_json(WORKSPACE_FORMAT).is_err());
    let mut project = self::project();
    project.minimum_reader_version = 2;
    let bytes = serde_json::to_vec(&project).unwrap();
    assert!(ProjectDocument::from_json(&bytes, WORKSPACE_FORMAT).is_err());
    assert!(ProjectDocument::from_json(&bytes, MANIFEST_FORMAT).is_err());
}

#[test]
fn drafts_can_be_incomplete_but_reviewed_contributors_must_meet_the_schema() {
    let mut project = project();
    project.data.schema.data.fields.push(FieldDefinition {
        id: "name".into(),
        label: "Display name".into(),
        field_type: FieldType::ShortText,
        visibility: Visibility::PrivateEncrypted,
        required: true,
        choices: vec![],
    });
    project.validate_as(WORKSPACE_FORMAT).unwrap();
    project.data.contributors[0].data.status = ContributorStatus::Approved;
    assert!(project.validate_as(WORKSPACE_FORMAT).is_err());
    project.data.contributors[0]
        .data
        .fields
        .insert("name".into(), serde_json::json!("A contributor"));
    project.validate_as(WORKSPACE_FORMAT).unwrap();
}
