//! Pure auxiliary time evidence preparation. No dependency on release or crypto.

use std::{
    collections::{BTreeMap, HashSet},
    future::Future,
    pin::Pin,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeScale {
    UnsmearedUtc,
    GoogleSmear,
    Local,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderClass {
    Nts,
    Advisory,
    Local,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provider {
    pub provider_id: String,
    pub operator_id: String,
    pub timescale: TimeScale,
    pub class: ProviderClass,
}

#[derive(Debug)]
pub struct TimeObservation {
    pub provider_id: String,
    pub operator_id: String,
    pub timescale: TimeScale,
    pub authenticated: bool,
    pub estimated_utc: SystemTime,
    pub uncertainty: Duration,
    pub rtt: Duration,
    pub monotonic_received_at: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    Verified,
    Good,
    Degraded,
    LocalOnly,
    Disagreement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    Metadata,
    Stale,
    HighRtt,
    Uncertain,
    Timescale,
    OperatorConflict,
    Outlier,
}

#[derive(Debug)]
pub struct RejectedObservation {
    pub provider_id: String,
    pub reason: Rejection,
}

#[derive(Clone, Copy, Debug)]
pub struct Estimate {
    pub utc: SystemTime,
    pub uncertainty: Duration,
    pub monotonic_at: Instant,
}

#[derive(Debug)]
pub struct Consensus {
    pub estimate: Option<Estimate>,
    pub confidence: Confidence,
    pub independent_operators: Vec<String>,
    pub rejected: Vec<RejectedObservation>,
}

#[derive(Debug, thiserror::Error)]
pub enum TimeError {
    #[error("time provider registry is invalid or too large")]
    Registry,
    #[error("time sample has invalid or overflowing arithmetic")]
    Arithmetic,
    #[error("time provider is unavailable or authentication failed")]
    Provider,
}

/// Created from reviewed adapters, never arbitrary project background URLs.
pub struct Registry {
    providers: BTreeMap<String, Provider>,
}

impl Registry {
    pub fn new(providers: Vec<Provider>) -> Result<Self, TimeError> {
        if providers.is_empty() || providers.len() > 16 {
            return Err(TimeError::Registry);
        }
        let mut map = BTreeMap::new();
        for provider in providers {
            if provider.provider_id.trim().is_empty()
                || provider.provider_id.len() > 128
                || provider.operator_id.trim().is_empty()
                || provider.operator_id.len() > 128
                || (provider.class == ProviderClass::Local)
                    != (provider.timescale == TimeScale::Local)
                || (provider.class == ProviderClass::Nts
                    && provider.timescale != TimeScale::UnsmearedUtc)
                || map.insert(provider.provider_id.clone(), provider).is_some()
            {
                return Err(TimeError::Registry);
            }
        }
        Ok(Self { providers: map })
    }

    pub fn evaluate(
        &self,
        observations: &[TimeObservation],
        now: Instant,
    ) -> Result<Consensus, TimeError> {
        if observations.len() > 32 {
            return Err(TimeError::Registry);
        }
        let mut rejected = Vec::new();
        let mut intervals = Vec::new();
        let mut seen = HashSet::new();
        for observation in observations {
            let reject = |reason| RejectedObservation {
                provider_id: observation.provider_id.clone(),
                reason,
            };
            let Some(provider) = self.providers.get(&observation.provider_id) else {
                rejected.push(reject(Rejection::Metadata));
                continue;
            };
            if !seen.insert(&observation.provider_id)
                || provider.operator_id != observation.operator_id
                || provider.timescale != observation.timescale
                || (provider.class == ProviderClass::Nts) != observation.authenticated
            {
                rejected.push(reject(Rejection::Metadata));
                continue;
            }
            if observation.timescale != TimeScale::UnsmearedUtc {
                rejected.push(reject(Rejection::Timescale));
                continue;
            }
            let Some(age) = now.checked_duration_since(observation.monotonic_received_at) else {
                rejected.push(reject(Rejection::Stale));
                continue;
            };
            if age > Duration::from_secs(15) {
                rejected.push(reject(Rejection::Stale));
                continue;
            }
            if observation.rtt > Duration::from_secs(2) {
                rejected.push(reject(Rejection::HighRtt));
                continue;
            }
            if observation.uncertainty > Duration::from_secs(5) {
                rejected.push(reject(Rejection::Uncertain));
                continue;
            }
            let midpoint = epoch_nanos(observation.estimated_utc)
                .checked_add(duration_nanos(age)?)
                .ok_or(TimeError::Arithmetic)?;
            let radius = duration_nanos(observation.uncertainty)?
                .checked_add(duration_nanos(observation.rtt)? / 2)
                .ok_or(TimeError::Arithmetic)?;
            intervals.push(Interval {
                provider: provider.clone(),
                low: midpoint.checked_sub(radius).ok_or(TimeError::Arithmetic)?,
                high: midpoint.checked_add(radius).ok_or(TimeError::Arithmetic)?,
            });
        }
        // Distinct hostnames never manufacture operator independence. Intersect
        // same-operator observations conservatively; exclude conflicted operators.
        let mut grouped: BTreeMap<String, Vec<Interval>> = BTreeMap::new();
        for interval in intervals {
            grouped
                .entry(interval.provider.operator_id.clone())
                .or_default()
                .push(interval);
        }
        let mut intervals = Vec::new();
        let mut authenticated_conflict = false;
        for mut group in grouped.into_values() {
            // Advisory aliases must not tighten an authenticated operator's
            // interval or change its uncertainty under an NTS label.
            if group.iter().any(|o| o.provider.class == ProviderClass::Nts) {
                group.retain(|o| {
                    if o.provider.class == ProviderClass::Nts {
                        true
                    } else {
                        rejected.push(RejectedObservation {
                            provider_id: o.provider.provider_id.clone(),
                            reason: Rejection::Metadata,
                        });
                        false
                    }
                });
            }
            let low = group
                .iter()
                .map(|o| o.low)
                .max()
                .ok_or(TimeError::Arithmetic)?;
            let high = group
                .iter()
                .map(|o| o.high)
                .min()
                .ok_or(TimeError::Arithmetic)?;
            let authenticated = group
                .iter()
                .filter(|o| o.provider.class == ProviderClass::Nts)
                .count()
                > 0;
            if low > high {
                authenticated_conflict |= authenticated;
                rejected.extend(group.into_iter().map(|o| RejectedObservation {
                    provider_id: o.provider.provider_id,
                    reason: Rejection::OperatorConflict,
                }));
                continue;
            }
            let mut chosen = group
                .into_iter()
                .max_by_key(|o| o.provider.class == ProviderClass::Nts)
                .ok_or(TimeError::Arithmetic)?;
            chosen.low = low;
            chosen.high = high;
            intervals.push(chosen);
        }
        // A shared intersection is required; a transitive chain of pairwise
        // overlaps cannot count as agreement. Prefer authenticated quorums.
        let mut best: Vec<&Interval> = Vec::new();
        let mut best_score = (0, 0);
        for point in intervals.iter().map(|o| o.low) {
            let cluster: Vec<_> = intervals
                .iter()
                .filter(|o| o.low <= point && o.high >= point)
                .collect();
            let score = (
                cluster
                    .iter()
                    .filter(|o| o.provider.class == ProviderClass::Nts)
                    .count(),
                cluster.len(),
            );
            if score > best_score {
                best_score = score;
                best = cluster;
            }
        }
        let authenticated_total = intervals
            .iter()
            .filter(|o| o.provider.class == ProviderClass::Nts)
            .count();
        let disagreement = authenticated_conflict || authenticated_total > best_score.0;
        let mut result = Consensus {
            estimate: None,
            confidence: if disagreement {
                Confidence::Disagreement
            } else {
                Confidence::LocalOnly
            },
            independent_operators: Vec::new(),
            rejected,
        };
        if best.len() < 2 || disagreement {
            return Ok(result);
        }
        let low = best
            .iter()
            .map(|o| o.low)
            .max()
            .ok_or(TimeError::Arithmetic)?;
        let high = best
            .iter()
            .map(|o| o.high)
            .min()
            .ok_or(TimeError::Arithmetic)?;
        let midpoint = low
            .checked_add((high - low) / 2)
            .ok_or(TimeError::Arithmetic)?;
        let radius = (midpoint - low).max(high - midpoint);
        result.estimate = Some(Estimate {
            utc: from_epoch_nanos(midpoint)?,
            uncertainty: nanos_duration(radius)?,
            monotonic_at: now,
        });
        result.confidence = match best_score.0 {
            0 => Confidence::Degraded,
            1 => Confidence::Good,
            _ => Confidence::Verified,
        };
        result.independent_operators = best
            .iter()
            .map(|o| o.provider.operator_id.clone())
            .collect();
        for interval in &intervals {
            if !best
                .iter()
                .any(|o| o.provider.operator_id == interval.provider.operator_id)
            {
                result.rejected.push(RejectedObservation {
                    provider_id: interval.provider.provider_id.clone(),
                    reason: Rejection::Outlier,
                });
            }
        }
        Ok(result)
    }
}

struct Interval {
    provider: Provider,
    low: i128,
    high: i128,
}

fn duration_nanos(duration: Duration) -> Result<i128, TimeError> {
    i128::try_from(duration.as_nanos()).map_err(|_| TimeError::Arithmetic)
}

fn epoch_nanos(time: SystemTime) -> i128 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(value) => value.as_nanos() as i128,
        Err(value) => -(value.duration().as_nanos() as i128),
    }
}

fn nanos_duration(nanos: i128) -> Result<Duration, TimeError> {
    let nanos = u128::try_from(nanos).map_err(|_| TimeError::Arithmetic)?;
    Ok(Duration::new(
        u64::try_from(nanos / 1_000_000_000).map_err(|_| TimeError::Arithmetic)?,
        (nanos % 1_000_000_000) as u32,
    ))
}

fn from_epoch_nanos(nanos: i128) -> Result<SystemTime, TimeError> {
    let duration = nanos_duration(nanos.checked_abs().ok_or(TimeError::Arithmetic)?)?;
    if nanos < 0 {
        UNIX_EPOCH.checked_sub(duration)
    } else {
        UNIX_EPOCH.checked_add(duration)
    }
    .ok_or(TimeError::Arithmetic)
}

#[derive(Debug)]
pub struct ClockReading {
    pub utc: SystemTime,
    pub uncertainty: Duration,
    pub confidence: Confidence,
    pub clock_jump: bool,
    pub query_due: bool,
}

pub struct TimeEngine {
    registry: Registry,
    anchor: Option<Estimate>,
    confidence: Confidence,
    last_wall: Option<(SystemTime, Instant)>,
    last_query: Option<Instant>,
    force_query: bool,
    clock_jump: bool,
}

impl TimeEngine {
    pub fn new(registry: Registry) -> Self {
        Self {
            registry,
            anchor: None,
            confidence: Confidence::LocalOnly,
            last_wall: None,
            last_query: None,
            force_query: true,
            clock_jump: false,
        }
    }

    pub fn accept_observations(
        &mut self,
        observations: &[TimeObservation],
        now: Instant,
    ) -> Result<Consensus, TimeError> {
        let report = self.registry.evaluate(observations, now)?;
        if let Some(estimate) = report.estimate {
            self.anchor = Some(estimate);
        }
        self.confidence = report.confidence;
        self.last_query = Some(now);
        self.force_query = false;
        if report.estimate.is_some() {
            self.clock_jump = false;
        }
        Ok(report)
    }

    pub fn on_reconnect(&mut self) {
        self.force_query = true;
    }

    pub fn on_resume(&mut self) {
        self.anchor = None;
        self.confidence = Confidence::LocalOnly;
        self.last_wall = None;
        self.force_query = true;
    }

    pub fn read(
        &mut self,
        wall: SystemTime,
        now: Instant,
        remaining: Duration,
        minimized: bool,
    ) -> Result<ClockReading, TimeError> {
        if let Some((prior_wall, prior_monotonic)) = self.last_wall {
            if let Some(elapsed) = now.checked_duration_since(prior_monotonic) {
                let expected = epoch_nanos(prior_wall)
                    .checked_add(duration_nanos(elapsed)?)
                    .ok_or(TimeError::Arithmetic)?;
                let difference = epoch_nanos(wall)
                    .checked_sub(expected)
                    .ok_or(TimeError::Arithmetic)?;
                if difference.checked_abs().ok_or(TimeError::Arithmetic)? > 5_000_000_000 {
                    self.clock_jump = true;
                    self.force_query = true;
                }
            } else {
                self.on_resume();
            }
        }
        self.last_wall = Some((wall, now));
        let interval = if minimized || remaining > Duration::from_secs(24 * 3600) {
            Duration::from_secs(45 * 60)
        } else if remaining > Duration::from_secs(3600) {
            Duration::from_secs(10 * 60)
        } else if remaining > Duration::from_secs(60) {
            Duration::from_secs(2 * 60)
        } else {
            Duration::from_secs(60)
        };
        let query_due = self.force_query
            || self.last_query.is_none_or(|last| {
                now.checked_duration_since(last)
                    .is_none_or(|age| age >= interval)
            });
        if let Some(anchor) = self.anchor {
            if let Some(age) = now.checked_duration_since(anchor.monotonic_at) {
                let utc = anchor.utc.checked_add(age).ok_or(TimeError::Arithmetic)?;
                // Conservative 50 ppm drift bound. Never claim infinite precision.
                let drift = nanos_duration(duration_nanos(age)? / 20_000)?;
                let uncertainty = anchor
                    .uncertainty
                    .checked_add(drift)
                    .ok_or(TimeError::Arithmetic)?;
                let confidence = if age > Duration::from_secs(2 * 3600)
                    && self.confidence != Confidence::Disagreement
                {
                    Confidence::LocalOnly
                } else {
                    self.confidence
                };
                return Ok(ClockReading {
                    utc,
                    uncertainty,
                    confidence,
                    clock_jump: self.clock_jump,
                    query_due,
                });
            }
            self.on_resume();
        }
        Ok(ClockReading {
            utc: wall,
            uncertainty: Duration::from_secs(24 * 3600),
            confidence: self.confidence,
            clock_jump: self.clock_jump,
            query_due,
        })
    }
}

pub type ObservationFuture<'a> =
    Pin<Box<dyn Future<Output = Result<TimeObservation, TimeError>> + Send + 'a>>;

pub trait TimeProvider: Send + Sync {
    fn metadata(&self) -> &Provider;
    fn observe(&self) -> ObservationFuture<'_>;
}

#[cfg(test)]
mod tests;
