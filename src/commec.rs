use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CommecOut {
    pub ran: bool,
    pub flag: bool,
    pub raw: Value,
}

pub fn run(db: Option<&Path>, bin: Option<&str>, fasta_path: &Path) -> CommecOut {
    let Some(db) = db else {
        return CommecOut {
            ran: false,
            flag: false,
            raw: json!({"status": "absent"}),
        };
    };
    let bin = bin.unwrap_or("commec");
    let out_dir = fasta_path.parent().unwrap_or(Path::new("."));
    let screen = Command::new(bin)
        .args(["screen", "-d"])
        .arg(db)
        .arg("-o")
        .arg(out_dir)
        .arg(fasta_path)
        .output();
    match screen {
        Ok(o) if o.status.success() => {
            let flag = Command::new(bin).args(["flag"]).arg(out_dir).output();
            let text = flag
                .ok()
                .map(|f| String::from_utf8_lossy(&f.stdout).into_owned())
                .unwrap_or_default();
            let flagged = text.to_ascii_lowercase().contains("flag")
                && !text.to_ascii_lowercase().contains("no flag");
            CommecOut {
                ran: true,
                flag: flagged,
                raw: json!({"stdout": text}),
            }
        }
        Ok(o) => CommecOut {
            ran: false,
            flag: false,
            raw: json!({
                "status": "failed",
                "stderr": String::from_utf8_lossy(&o.stderr)
            }),
        },
        Err(e) => CommecOut {
            ran: false,
            flag: false,
            raw: json!({"status": "spawn_error", "error": e.to_string()}),
        },
    }
}
