use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagParse {
    Cleared,
    Flagged,
    Unparsed,
}

/// Parse `commec flag` stdout without spawning a process.
pub fn parse_flag_stdout(stdout: &str) -> FlagParse {
    let mut saw_line = false;
    let mut any_flagged = false;
    let mut any_unparsed = false;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        saw_line = true;
        let rubric = rubric_field(line);
        let norm = rubric.to_ascii_lowercase();
        if norm == "flag" {
            any_flagged = true;
        } else if norm == "no flag" {
            // cleared for this line
        } else {
            any_unparsed = true;
        }
    }

    if !saw_line {
        return FlagParse::Unparsed;
    }
    if any_unparsed {
        return FlagParse::Unparsed;
    }
    if any_flagged {
        return FlagParse::Flagged;
    }
    FlagParse::Cleared
}

fn rubric_field(line: &str) -> &str {
    match line.find('\t') {
        Some(i) => line[i + 1..].trim(),
        None => line.trim(),
    }
}

#[derive(Debug, Clone)]
pub struct CommecOut {
    pub ran: bool,
    pub flag: bool,
    pub raw: Value,
}

fn from_parse(stdout: String, parse: FlagParse) -> CommecOut {
    let status = match parse {
        FlagParse::Cleared => "cleared",
        FlagParse::Flagged => "flagged",
        FlagParse::Unparsed => "unparsed",
    };
    match parse {
        FlagParse::Cleared => CommecOut {
            ran: true,
            flag: false,
            raw: json!({"stdout": stdout, "status": status}),
        },
        FlagParse::Flagged => CommecOut {
            ran: true,
            flag: true,
            raw: json!({"stdout": stdout, "status": status}),
        },
        FlagParse::Unparsed => CommecOut {
            ran: false,
            flag: false,
            raw: json!({"stdout": stdout, "status": status}),
        },
    }
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
            match flag {
                Ok(f) if f.status.success() => {
                    let text = String::from_utf8_lossy(&f.stdout).into_owned();
                    if text.is_empty() {
                        return CommecOut {
                            ran: false,
                            flag: false,
                            raw: json!({"stdout": text, "status": "unparsed"}),
                        };
                    }
                    let parse = parse_flag_stdout(&text);
                    from_parse(text, parse)
                }
                Ok(f) => {
                    let text = String::from_utf8_lossy(&f.stdout).into_owned();
                    CommecOut {
                        ran: false,
                        flag: false,
                        raw: json!({
                            "stdout": text,
                            "status": "unparsed",
                            "stderr": String::from_utf8_lossy(&f.stderr)
                        }),
                    }
                }
                Err(e) => CommecOut {
                    ran: false,
                    flag: false,
                    raw: json!({"status": "spawn_error", "error": e.to_string()}),
                },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reduce::{self, EngineSignals};
    use crate::pattern::PatternHit;
    use crate::spec::IntendedFunction;
    use crate::verdict::Policy;

    #[test]
    fn parse_cleared() {
        assert_eq!(
            parse_flag_stdout("construct.fa\tNo Flag\n"),
            FlagParse::Cleared
        );
    }

    #[test]
    fn parse_uncleared() {
        assert_eq!(parse_flag_stdout("construct.fa\tFlag\n"), FlagParse::Flagged);
    }

    #[test]
    fn parse_empty_unavailable() {
        assert_eq!(parse_flag_stdout(""), FlagParse::Unparsed);
    }

    #[test]
    fn parse_substring_trap_unavailable() {
        assert_eq!(
            parse_flag_stdout("something was flagged earlier but No Flag\n"),
            FlagParse::Unparsed
        );
    }

    #[test]
    fn parse_warning_unavailable() {
        assert_eq!(
            parse_flag_stdout("construct.fa\tWarning\n"),
            FlagParse::Unparsed
        );
    }

    #[test]
    fn parse_mixed_uncleared_wins() {
        assert_eq!(
            parse_flag_stdout("construct.fa\tNo Flag\nother\tFlag\n"),
            FlagParse::Flagged
        );
    }

    #[test]
    fn non_zero_flag_exit_escalates_not_release() {
        let parse = FlagParse::Unparsed;
        let out = from_parse(String::new(), parse);
        assert!(!out.ran);
        let (_, p, _) = reduce::reduce(&EngineSignals {
            commec_required: true,
            commec_ran: out.ran,
            commec_flag: out.flag,
            pattern: PatternHit {
                repeat_array: false,
                rt_like_cds: false,
                programmable_system_shape: false,
            },
            intended: IntendedFunction::Reporter,
            ..Default::default()
        });
        assert_eq!(p, Policy::Escalate);
    }
}
