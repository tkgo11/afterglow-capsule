//! Isolated experimental quality policy; no production render dependency.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Quality {
    Full,
    Reduced,
    Static,
    Opaque,
}

impl Quality {
    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Reduced => "reduced",
            Self::Static => "static",
            Self::Opaque => "opaque",
        }
    }

    pub fn shader_value(self) -> f32 {
        match self {
            Self::Full => 0.0,
            Self::Reduced => 1.0,
            Self::Static => 2.0,
            Self::Opaque => 3.0,
        }
    }

    // Degrade only in this spike, so slow samples cannot cause mode oscillation.
    // Explicit Reduced Transparency always overrides automatic quality.
    pub fn degrade(self, frame_ms: f64) -> Self {
        match self {
            Self::Opaque | Self::Static => self,
            _ if !frame_ms.is_finite() || frame_ms > 25.0 => Self::Static,
            Self::Full if frame_ms > 16.7 => Self::Reduced,
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_frames_degrade_and_do_not_oscillate() {
        assert_eq!(Quality::Full.degrade(16.0), Quality::Full);
        assert_eq!(Quality::Full.degrade(20.0), Quality::Reduced);
        assert_eq!(Quality::Reduced.degrade(30.0), Quality::Static);
        assert_eq!(Quality::Static.degrade(2.0), Quality::Static);
        assert_eq!(Quality::Full.degrade(f64::NAN), Quality::Static);
    }

    #[test]
    fn accessibility_choice_survives_fast_and_slow_samples() {
        for milliseconds in [1.0, 20.0, 100.0, f64::INFINITY] {
            assert_eq!(Quality::Opaque.degrade(milliseconds), Quality::Opaque);
        }
    }

    #[test]
    fn shader_is_valid_wgsl() {
        let module = naga::front::wgsl::parse_str(include_str!("glass.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap();
    }
}
