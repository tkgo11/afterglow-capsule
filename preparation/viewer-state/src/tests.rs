use super::*;
use ag_prepared_capsule::{CapsuleAssembler, serialize};
use ag_prepared_object_crypto::CompressionPolicy;
#[path = "../../test_support.rs"]
mod fixtures;

struct ProjectFixture {
    bytes: Vec<u8>,
    entry_id: StableId,
    image_id: StableId,
}

fn project(round: u64, index_change: impl FnOnce(&mut Document<ArchiveIndex>)) -> ProjectFixture {
    let mut manifest = fixtures::manifest();
    manifest.release = fixtures::release(round);
    let profile_id = StableId::random();
    let contributor_id = StableId::random();
    let entry_id = StableId::random();
    let entry_object = StableId::random();
    let image_id = StableId::random();
    let mut archive = Document::new(
        ARCHIVE_FORMAT,
        ArchiveIndex {
            project_id: manifest.project_id,
            build_id: manifest.build_id,
            contributors: vec![ContributorIndex {
                contributor_id,
                profile_object_id: profile_id,
            }],
            entries: vec![EntryIndex {
                entry_id,
                contributor_id,
                object_id: entry_object,
            }],
        },
    )
    .unwrap();
    index_change(&mut archive);
    let index_bytes = serde_json::to_vec(&archive).unwrap();
    let entry = Document::new(
        ENTRY_FORMAT,
        EntryContent {
            title: "Protected public fixture".into(),
            blocks: vec![ContentBlock::Image {
                object_id: image_id,
                alt: "Public fixture image".into(),
            }],
        },
    )
    .unwrap();
    let private_id = manifest.private_manifest_id;
    let mut assembler = CapsuleAssembler::new(manifest, Limits::default()).unwrap();
    assembler
        .add_private(
            private_id,
            ObjectClass::PrivateJson,
            &index_bytes,
            CompressionPolicy::Text,
        )
        .unwrap();
    assembler
        .add_private(
            profile_id,
            ObjectClass::PrivateJson,
            b"{}",
            CompressionPolicy::Text,
        )
        .unwrap();
    assembler
        .add_private(
            entry_object,
            ObjectClass::PrivateJson,
            &entry.to_json(ENTRY_FORMAT).unwrap(),
            CompressionPolicy::Text,
        )
        .unwrap();
    assembler
        .add_private(
            image_id,
            ObjectClass::PrivateImage,
            b"public synthetic image data",
            CompressionPolicy::None,
        )
        .unwrap();
    let bytes = assembler
        .finish(
            &semver::Version::new(0, 1, 0),
            &[b"Protected public fixture"],
        )
        .unwrap();
    ProjectFixture {
        bytes,
        entry_id,
        image_id,
    }
}

fn session(bytes: &[u8]) -> ViewerSession<'_> {
    ViewerSession::load(bytes, &semver::Version::new(0, 1, 0), Limits::default()).unwrap()
}

fn verified() -> FetchOutcome {
    FetchOutcome::Verified(
        QuicknetEngine::new(fixtures::release(1000))
            .unwrap()
            .verify_beacon(&fixtures::beacon())
            .unwrap(),
    )
}

fn release(session: &mut ViewerSession<'_>) {
    session.start().unwrap();
    session.begin_release_check().unwrap();
    session.apply_release_result(verified()).unwrap();
    session.start_ceremony().unwrap();
    session.complete_ceremony().unwrap();
    session.enter_archive().unwrap();
}

#[test]
fn valid_release_authenticates_before_ceremony_and_navigation() {
    let project = project(1000, |_| {});
    let mut session = session(&project.bytes);
    release(&mut session);
    assert_eq!(
        session
            .history()
            .iter()
            .take(7)
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            State::Boot,
            State::PreRelease,
            State::ReleaseMaterialCheck,
            State::Unlocking,
            State::Authenticating,
            State::ReadyForCeremony,
            State::Ceremony
        ]
    );
    session.open_entry(project.entry_id).unwrap();
    let entry = session.read_entry().unwrap();
    assert_eq!(entry.content().title, "Protected public fixture");
    assert_eq!(format!("{entry:?}"), "SecretEntry([REDACTED])");
    session.open_media(project.image_id).unwrap();
    assert_eq!(
        &*session.read_media().unwrap(),
        b"public synthetic image data"
    );
    session.back().unwrap();
    assert_eq!(session.state(), &State::EntryDetail(project.entry_id));
    session.back().unwrap();
    assert_eq!(session.state(), &State::PostReleaseHome);
}

#[test]
fn ui_actions_and_unavailable_material_cannot_unlock_or_reveal_private_content() {
    let project = project(1000, |_| {});
    let mut session = session(&project.bytes);
    assert!(session.apply_release_result(verified()).is_err());
    for _ in 0..2 {
        assert!(session.start_ceremony().is_err());
        assert!(session.complete_ceremony().is_err());
        assert!(session.enter_archive().is_err());
        assert!(session.open_entry(project.entry_id).is_err());
        assert!(session.read_entry().is_err());
        assert!(session.read_media().is_err());
        if session.state() == &State::Boot {
            session.start().unwrap();
        }
    }
    session.begin_release_check().unwrap();
    session
        .apply_release_result(FetchOutcome::Waiting(vec![]))
        .unwrap();
    assert_eq!(session.state(), &State::PreRelease);
    assert!(matches!(
        session.begin_release_check(),
        Err(SessionError::RateLimited)
    ));
    assert!(session.cek.is_none());
}

#[test]
fn a_verified_beacon_for_another_round_still_fails_closed() {
    let project = project(1001, |_| {});
    let mut session = session(&project.bytes);
    session.start().unwrap();
    session.begin_release_check().unwrap();
    assert!(session.apply_release_result(verified()).is_err());
    assert_eq!(
        session.state(),
        &State::LockedError(Failure::ReleaseIntegrity)
    );
    assert!(session.cek.is_none());
    assert!(session.archive.is_none());
}

#[test]
fn private_index_version_and_project_binding_fail_closed_after_real_decryption() {
    for project in [
        project(1000, |d| d.format_version = 2),
        project(1000, |d| d.data.project_id = StableId::random()),
    ] {
        let mut session = session(&project.bytes);
        session.start().unwrap();
        session.begin_release_check().unwrap();
        assert!(session.apply_release_result(verified()).is_err());
        assert_eq!(session.state(), &State::LockedError(Failure::PrivateFormat));
        assert!(session.cek.is_none());
    }
}

#[test]
fn maliciously_rehashed_private_corruption_cannot_reach_ready_or_ceremony() {
    let project = project(1000, |_| {});
    let loaded = LoadedCapsule::parse(
        &project.bytes,
        &semver::Version::new(0, 1, 0),
        Limits::default(),
    )
    .unwrap();
    let header = ag_capsule::CapsuleHeader::parse(&project.bytes).unwrap();
    let public = header
        .section(
            &project.bytes,
            ag_capsule::SectionKind::PublicStore,
            u64::MAX,
        )
        .unwrap();
    let mut private = header
        .section(
            &project.bytes,
            ag_capsule::SectionKind::PrivateStore,
            u64::MAX,
        )
        .unwrap()
        .to_vec();
    let metadata = loaded
        .manifest()
        .objects
        .iter()
        .find(|o| o.object_id == loaded.manifest().private_manifest_id)
        .unwrap();
    private[(metadata.offset + metadata.length - 1) as usize] ^= 1;
    let bytes = serialize(
        loaded.manifest(),
        public,
        &private,
        loaded.envelope(),
        Limits::default(),
    )
    .unwrap();
    let mut session = session(&bytes);
    session.start().unwrap();
    session.begin_release_check().unwrap();
    assert!(session.apply_release_result(verified()).is_err());
    assert_eq!(
        session.state(),
        &State::LockedError(Failure::ObjectIntegrity)
    );
    assert!(session.start_ceremony().is_err());
    assert!(session.cek.is_none());
}

#[test]
fn ceremony_requires_explicit_completion_and_enter_and_navigation_is_bounded() {
    let project = project(1000, |_| {});
    let mut session = session(&project.bytes);
    session.start().unwrap();
    session.begin_release_check().unwrap();
    session.apply_release_result(verified()).unwrap();
    assert!(session.enter_archive().is_err());
    session.start_ceremony().unwrap();
    assert!(session.enter_archive().is_err());
    session.complete_ceremony().unwrap();
    session.enter_archive().unwrap();
    assert!(session.open_entry(StableId::random()).is_err());
    session.open_entry(project.entry_id).unwrap();
    assert!(session.open_media(StableId::random()).is_err());
    for _ in 0..100 {
        session.back().unwrap();
        session.open_entry(project.entry_id).unwrap();
    }
    assert!(session.history().len() <= 64);
}

#[test]
fn future_or_past_wall_clock_estimates_do_not_grant_release_capability() {
    use ag_prepared_time::{Confidence, Provider, ProviderClass, Registry, TimeEngine, TimeScale};
    let project = project(1000, |_| {});
    for future in [true, false] {
        let registry = Registry::new(vec![Provider {
            provider_id: "local".into(),
            operator_id: "local".into(),
            timescale: TimeScale::Local,
            class: ProviderClass::Local,
        }])
        .unwrap();
        let mut time = TimeEngine::new(registry);
        let wall = std::time::SystemTime::now();
        let wall = if future {
            wall + Duration::from_secs(86400)
        } else {
            wall - Duration::from_secs(86400)
        };
        let reading = time
            .read(wall, Instant::now(), Duration::ZERO, false)
            .unwrap();
        assert_eq!(reading.utc, wall);
        assert_eq!(reading.confidence, Confidence::LocalOnly);
        let mut session = session(&project.bytes);
        session.start().unwrap();
        session.begin_release_check().unwrap();
        session
            .apply_release_result(FetchOutcome::Waiting(vec![]))
            .unwrap();
        assert_eq!(session.state(), &State::PreRelease);
        assert!(session.start_ceremony().is_err());
        assert!(session.archive_index().is_err());
        assert!(session.cek.is_none());
    }
}
