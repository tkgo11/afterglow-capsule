//! Bounded, duplicate-rejecting evidence inputs; no networking or trust discovery.
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    fmt, fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, String>;
pub const JSON_LIMIT: u64 = 2 * 1024 * 1024;
pub const LOG_LIMIT: u64 = 16 * 1024 * 1024;
pub const EXE_LIMIT: u64 = 256 * 1024 * 1024;

struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON without duplicate keys")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Bool(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::from(v)))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::from(v)))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Unique, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| Unique(Value::Number(n)))
                    .ok_or_else(|| E::custom("non-finite number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::from(v)))
            }
            fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::from(v)))
            }
            fn visit_unit<E: de::Error>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut values = Vec::new();
                while let Some(v) = a.next_element::<Unique>()? {
                    values.push(v.0);
                }
                Ok(Unique(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut values = Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if values.contains_key(&k) {
                        return Err(de::Error::custom(format!("duplicate JSON field: {k}")));
                    }
                    values.insert(k, a.next_value::<Unique>()?.0);
                }
                Ok(Unique(Value::Object(values)))
            }
        }
        d.deserialize_any(V)
    }
}
pub fn parse(bytes: &[u8]) -> Result<Value> {
    // Windows PowerShell 5.1 writes UTF-8 BOM; it has no semantic significance.
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let mut d = serde_json::Deserializer::from_slice(bytes);
    let v = Unique::deserialize(&mut d)
        .map_err(|e| format!("invalid JSON: {e}"))?
        .0;
    d.end().map_err(|e| format!("trailing JSON: {e}"))?;
    Ok(v)
}
pub fn safe_path(root: &Path, relative: &str) -> Result<PathBuf> {
    if relative.is_empty() || relative.contains('\\') || relative.contains(':') {
        return Err("unsafe relative evidence path".into());
    }
    let rel = Path::new(relative);
    if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err("evidence path traversal is prohibited".into());
    }
    let mut path = root.to_path_buf();
    for ancestor in root.ancestors() {
        if let Ok(metadata) = fs::symlink_metadata(ancestor)
            && metadata.file_type().is_symlink()
        {
            return Err("symlink evidence root/ancestor prohibited".into());
        }
    }
    for c in rel.components() {
        path.push(c);
        if let Ok(metadata) = fs::symlink_metadata(&path)
            && metadata.file_type().is_symlink()
        {
            return Err("symlink evidence path prohibited".into());
        }
    }
    Ok(path)
}
pub fn read(root: &Path, relative: &str, max: u64) -> Result<Vec<u8>> {
    let path = safe_path(root, relative)?;
    let metadata = fs::metadata(&path).map_err(|e| format!("{relative}: {e}"))?;
    if !metadata.is_file() || metadata.len() > max {
        return Err(format!("{relative}: invalid file or size limit"));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max {
        return Err(format!("{relative}: size limit"));
    }
    Ok(bytes)
}
pub fn json(root: &Path, relative: &str) -> Result<Value> {
    parse(&read(root, relative, JSON_LIMIT)?)
}
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("missing/non-string {key}"))
}
pub fn integer(v: &Value, key: &str) -> Result<u64> {
    v.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing/non-integer {key}"))
}
pub fn array<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    v.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing/non-array {key}"))
}
pub fn yes(v: &Value, key: &str) -> Result<()> {
    if v.get(key) == Some(&Value::Bool(true)) {
        Ok(())
    } else {
        Err(format!("{key} must be true"))
    }
}
pub fn no(v: &Value, key: &str) -> Result<()> {
    if v.get(key) == Some(&Value::Bool(false)) {
        Ok(())
    } else {
        Err(format!("{key} must be false"))
    }
}
pub fn same(a: &Value, b: &Value, key: &str) -> Result<()> {
    let x = a.get(key).ok_or_else(|| format!("missing {key}"))?;
    let y = b
        .get(key)
        .ok_or_else(|| format!("missing expected {key}"))?;
    if x == y {
        Ok(())
    } else {
        Err(format!("{key} mismatch"))
    }
}
pub fn hex(v: &str, size: usize) -> Result<()> {
    if v.len() == size && v.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(format!("invalid {size}-character hex"))
    }
}
pub fn digest(root: &Path, path: &str, expected: &str, max: u64) -> Result<Vec<u8>> {
    hex(expected, 64)?;
    let bytes = read(root, path, max)?;
    if hash(&bytes).eq_ignore_ascii_case(expected) {
        Ok(bytes)
    } else {
        Err(format!("{path}: SHA-256 mismatch"))
    }
}
pub fn document(v: &Value, name: &str) -> Result<()> {
    if text(v, "format_name")? != name
        || integer(v, "format_version")? != 2
        || integer(v, "minimum_reader_version")? != 2
    {
        return Err("unsupported/obsolete evidence format; version 2 is required".into());
    }
    Ok(())
}
pub fn provenance(v: &Value) -> Result<()> {
    if text(v, "repository")? != "tkgo11/afterglow-capsule"
        || text(v, "workflow_path")? != ".github/workflows/spikes.yml"
    {
        return Err("unexpected workflow repository/path".into());
    }
    hex(text(v, "source_commit")?, 40)?;
    if integer(v, "workflow_run_id")? == 0 || integer(v, "workflow_run_attempt")? == 0 {
        return Err("invalid workflow run provenance".into());
    }
    Ok(())
}
