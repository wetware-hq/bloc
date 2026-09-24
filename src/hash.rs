use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex_encode(&h.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Canonical JSON: sorted object keys, no insignificant whitespace.
pub fn canonical_json(v: &Value) -> Vec<u8> {
    fn write(v: &Value, out: &mut Vec<u8>) {
        match v {
            Value::Null => out.extend_from_slice(b"null"),
            Value::Bool(true) => out.extend_from_slice(b"true"),
            Value::Bool(false) => out.extend_from_slice(b"false"),
            Value::Number(n) => out.extend_from_slice(n.to_string().as_bytes()),
            Value::String(s) => out.extend_from_slice(serde_json::to_string(s).unwrap().as_bytes()),
            Value::Array(a) => {
                out.push(b'[');
                for (i, x) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(b',');
                    }
                    write(x, out);
                }
                out.push(b']');
            }
            Value::Object(map) => {
                out.push(b'{');
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        out.push(b',');
                    }
                    out.extend_from_slice(serde_json::to_string(*k).unwrap().as_bytes());
                    out.push(b':');
                    write(&map[*k], out);
                }
                out.push(b'}');
            }
        }
    }
    let mut out = Vec::new();
    write(v, &mut out);
    out
}

pub fn spec_hash(spec_json: &Value) -> String {
    sha256_hex(&canonical_json(spec_json))
}

pub fn fasta_hash(fasta: &str) -> String {
    sha256_hex(fasta.as_bytes())
}

pub fn identity_key(spec_sha: &str, fasta_sha: &str) -> String {
    sha256_hex(format!("{spec_sha}\x1e{fasta_sha}").as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_is_stable() {
        let a = json!({"b": 1, "a": 2});
        let b = json!({"a": 2, "b": 1});
        assert_eq!(canonical_json(&a), canonical_json(&b));
        assert_eq!(spec_hash(&a), spec_hash(&b));
    }
}
