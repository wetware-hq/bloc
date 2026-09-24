use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rubric {
    #[serde(rename = "No Flag")]
    NoFlag,
    Flag,
    Undefined,
}

impl Rubric {
    pub fn word(self) -> &'static str {
        match self {
            Rubric::NoFlag => "No Flag",
            Rubric::Flag => "Flag",
            Rubric::Undefined => "Undefined",
        }
    }
}

impl std::fmt::Display for Rubric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.word())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Policy {
    Release,
    Hold,
    Escalate,
}

impl Policy {
    pub fn exit_code(self) -> i32 {
        match self {
            Policy::Release => 0,
            Policy::Hold => 10,
            Policy::Escalate => 20,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Policy::Release => "RELEASE",
            Policy::Hold => "HOLD",
            Policy::Escalate => "ESCALATE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reason {
    pub code: String,
    pub clause: String,
    pub engine: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub spec_sha256: String,
    pub fasta_sha256: String,
    pub identity: String,
    pub inhibit_version: String,
    pub engine_revs: serde_json::Value,
    pub utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    pub verdict_version: String,
    pub construct_id: String,
    pub rubric: Rubric,
    pub policy: Policy,
    pub reasons: Vec<Reason>,
    pub engines: serde_json::Value,
    pub receipt: Receipt,
}
