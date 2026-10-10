use super::*;

fn provider(id: &str, operator: &str, class: ProviderClass, timescale: TimeScale) -> Provider {
    Provider {
        provider_id: id.into(),
        operator_id: operator.into(),
        class,
        timescale,
    }
}

fn registry() -> Registry {
    Registry::new(vec![
        provider("a", "one", ProviderClass::Nts, TimeScale::UnsmearedUtc),
        provider("b", "two", ProviderClass::Nts, TimeScale::UnsmearedUtc),
        provider("alias", "one", ProviderClass::Nts, TimeScale::UnsmearedUtc),
        provider(
            "n",
            "three",
            ProviderClass::Advisory,
            TimeScale::UnsmearedUtc,
        ),
        provider(
            "n2",
            "four",
            ProviderClass::Advisory,
            TimeScale::UnsmearedUtc,
        ),
        provider(
            "g",
            "google",
            ProviderClass::Advisory,
            TimeScale::GoogleSmear,
        ),
    ])
    .unwrap()
}

fn sample(id: &str, offset_ms: i64, now: Instant) -> TimeObservation {
    let (operator, authenticated, timescale) = match id {
        "a" | "alias" => ("one", true, TimeScale::UnsmearedUtc),
        "b" => ("two", true, TimeScale::UnsmearedUtc),
        "n" => ("three", false, TimeScale::UnsmearedUtc),
        "n2" => ("four", false, TimeScale::UnsmearedUtc),
        _ => ("google", false, TimeScale::GoogleSmear),
    };
    let utc = UNIX_EPOCH + Duration::from_secs(2_000_000_000);
    let delta = Duration::from_millis(offset_ms.unsigned_abs());
    TimeObservation {
        provider_id: id.into(),
        operator_id: operator.into(),
        authenticated,
        timescale,
        estimated_utc: if offset_ms >= 0 {
            utc + delta
        } else {
            utc - delta
        },
        uncertainty: Duration::from_millis(10),
        rtt: Duration::from_millis(20),
        monotonic_received_at: now,
    }
}

#[test]
fn independent_authenticated_quorums_good_and_degraded_are_distinct() {
    let now = Instant::now();
    for (ids, expected) in [
        (["a", "b"], Confidence::Verified),
        (["a", "n"], Confidence::Good),
        (["n", "n2"], Confidence::Degraded),
    ] {
        let report = registry()
            .evaluate(&[sample(ids[0], 0, now), sample(ids[1], 5, now)], now)
            .unwrap();
        assert_eq!(report.confidence, expected);
        assert_eq!(report.independent_operators.len(), 2);
    }
}

#[test]
fn aliases_and_isolated_sources_cannot_manufacture_a_quorum() {
    let now = Instant::now();
    let report = registry()
        .evaluate(&[sample("a", 0, now), sample("alias", 0, now)], now)
        .unwrap();
    assert_eq!(report.confidence, Confidence::LocalOnly);
    assert!(report.estimate.is_none());
    assert_eq!(
        registry()
            .evaluate(&[sample("a", 0, now)], now)
            .unwrap()
            .confidence,
        Confidence::LocalOnly
    );
}

#[test]
fn authenticated_conflicts_never_become_verified_even_with_a_majority() {
    let now = Instant::now();
    let report = registry()
        .evaluate(
            &[
                sample("a", 0, now),
                sample("b", 1000, now),
                sample("n", 0, now),
            ],
            now,
        )
        .unwrap();
    assert_eq!(report.confidence, Confidence::Disagreement);
    assert!(report.estimate.is_none());
    let report = registry()
        .evaluate(
            &[
                sample("a", 0, now),
                sample("alias", 1000, now),
                sample("b", 0, now),
            ],
            now,
        )
        .unwrap();
    assert_eq!(report.confidence, Confidence::Disagreement);
}

#[test]
fn google_smear_is_never_averaged_into_unsmeared_consensus() {
    let now = Instant::now();
    let report = registry()
        .evaluate(
            &[
                sample("a", 0, now),
                sample("b", 0, now),
                sample("g", 500, now),
            ],
            now,
        )
        .unwrap();
    assert_eq!(report.confidence, Confidence::Verified);
    assert_eq!(report.rejected[0].reason, Rejection::Timescale);
    assert_eq!(
        report.estimate.unwrap().utc,
        sample("a", 0, now).estimated_utc
    );
}

#[test]
fn high_rtt_stale_metadata_spoofing_and_advisory_outliers_are_rejected() {
    let now = Instant::now();
    let mut high = sample("b", 0, now);
    high.rtt = Duration::from_secs(3);
    let mut spoof = sample("n", 0, now);
    spoof.authenticated = true;
    let stale = sample("n2", 0, now - Duration::from_secs(16));
    let report = registry()
        .evaluate(&[sample("a", 0, now), high, spoof, stale], now)
        .unwrap();
    assert_eq!(report.confidence, Confidence::LocalOnly);
    assert_eq!(report.rejected.len(), 3);
    let report = registry()
        .evaluate(
            &[
                sample("a", 0, now),
                sample("b", 0, now),
                sample("n", 2000, now),
            ],
            now,
        )
        .unwrap();
    assert_eq!(report.confidence, Confidence::Verified);
    assert!(
        report
            .rejected
            .iter()
            .any(|o| o.reason == Rejection::Outlier)
    );
}

#[test]
fn monotonic_anchor_ignores_both_directions_of_wall_clock_jump() {
    let now = Instant::now();
    let original = sample("a", 0, now).estimated_utc;
    for jump in [
        original + Duration::from_secs(86400),
        original - Duration::from_secs(86400),
    ] {
        let mut engine = TimeEngine::new(registry());
        engine
            .accept_observations(&[sample("a", 0, now), sample("b", 0, now)], now)
            .unwrap();
        engine
            .read(original, now, Duration::from_secs(1000), false)
            .unwrap();
        let reading = engine
            .read(
                jump,
                now + Duration::from_secs(10),
                Duration::from_secs(1000),
                false,
            )
            .unwrap();
        assert_eq!(reading.utc, original + Duration::from_secs(10));
        assert!(reading.clock_jump);
        assert!(reading.query_due);
    }
}

#[test]
fn outage_uses_last_anchor_resume_invalidates_it_and_reconnect_requeries() {
    let now = Instant::now();
    let original = sample("a", 0, now).estimated_utc;
    let mut engine = TimeEngine::new(registry());
    engine
        .accept_observations(&[sample("a", 0, now), sample("b", 0, now)], now)
        .unwrap();
    engine
        .accept_observations(&[], now + Duration::from_secs(1))
        .unwrap();
    let read = engine
        .read(
            original,
            now + Duration::from_secs(2),
            Duration::from_secs(86400),
            false,
        )
        .unwrap();
    assert_eq!(read.confidence, Confidence::LocalOnly);
    assert_eq!(read.utc, original + Duration::from_secs(2));
    engine.on_reconnect();
    assert!(
        engine
            .read(
                original,
                now + Duration::from_secs(2),
                Duration::ZERO,
                false
            )
            .unwrap()
            .query_due
    );
    engine.on_resume();
    assert!(engine.anchor.is_none());
    assert_eq!(
        engine
            .read(original, now, Duration::ZERO, false)
            .unwrap()
            .confidence,
        Confidence::LocalOnly
    );
}

#[test]
fn query_frequency_does_not_hammer_near_release_and_minimized_is_quiet() {
    let now = Instant::now();
    let wall = sample("a", 0, now).estimated_utc;
    let mut engine = TimeEngine::new(registry());
    engine.accept_observations(&[], now).unwrap();
    assert!(
        !engine
            .read(wall, now + Duration::from_secs(30), Duration::ZERO, false)
            .unwrap()
            .query_due
    );
    assert!(
        engine
            .read(wall, now + Duration::from_secs(61), Duration::ZERO, false)
            .unwrap()
            .query_due
    );
    let mut engine = TimeEngine::new(registry());
    engine.accept_observations(&[], now).unwrap();
    assert!(
        !engine
            .read(
                wall + Duration::from_secs(61),
                now + Duration::from_secs(61),
                Duration::ZERO,
                true
            )
            .unwrap()
            .query_due
    );
}

#[test]
fn no_network_is_local_only_and_a_timezone_presentation_change_has_no_effect() {
    let now = Instant::now();
    let wall = sample("a", 0, now).estimated_utc;
    let mut engine = TimeEngine::new(registry());
    let first = engine.read(wall, now, Duration::ZERO, false).unwrap();
    let second = engine.read(wall, now, Duration::ZERO, false).unwrap();
    assert_eq!(first.utc, second.utc);
    assert_eq!(first.confidence, Confidence::LocalOnly);
}

#[test]
fn unauthenticated_alias_cannot_shrink_authenticated_uncertainty() {
    let now = Instant::now();
    let registry = Registry::new(vec![
        provider("a", "one", ProviderClass::Nts, TimeScale::UnsmearedUtc),
        provider("b", "two", ProviderClass::Nts, TimeScale::UnsmearedUtc),
        provider("n", "one", ProviderClass::Advisory, TimeScale::UnsmearedUtc),
    ])
    .unwrap();
    let mut advisory = sample("n", 0, now);
    advisory.operator_id = "one".into();
    advisory.rtt = Duration::ZERO;
    advisory.uncertainty = Duration::ZERO;
    let report = registry
        .evaluate(&[sample("a", 0, now), sample("b", 0, now), advisory], now)
        .unwrap();
    assert_eq!(report.confidence, Confidence::Verified);
    assert_eq!(
        report.estimate.unwrap().uncertainty,
        Duration::from_millis(20)
    );
    assert_eq!(report.rejected.len(), 1);
}
