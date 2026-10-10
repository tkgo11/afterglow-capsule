use serde::{Deserialize, Deserializer, Serialize, de::Error};
use uuid::{Uuid, Variant, Version};

/// Opaque random identity; names never determine it. Accept UUIDv4/v7 only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct StableId(Uuid);

impl StableId {
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(value: &str) -> Result<Self, &'static str> {
        let uuid = Uuid::parse_str(value).map_err(|_| "invalid UUID")?;
        if uuid.get_variant() != Variant::RFC4122
            || !matches!(
                uuid.get_version(),
                Some(Version::Random | Version::SortRand)
            )
        {
            return Err("identity must be UUIDv4 or UUIDv7");
        }
        Ok(Self(uuid))
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl std::fmt::Display for StableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for StableId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}
