use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignSpec {
    pub spec_version: String,
    pub construct_id: String,
    pub designer: Designer,
    pub chassis: Chassis,
    pub intended_function: IntendedFunction,
    pub intended_bsl: Bsl,
    pub not_for_synthesis: bool,
    pub fragments: Vec<Fragment>,
    #[serde(default)]
    pub assembly: Option<Assembly>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Designer {
    pub agent: String,
    #[serde(default)]
    pub model: Option<String>,
    pub human: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Chassis {
    None,
    #[serde(rename = "e_coli_k12")]
    EColiK12,
    #[serde(rename = "e_coli_b")]
    EColiB,
    Yeast,
    CellFree,
    OtherBsl1,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntendedFunction {
    Reporter,
    Housekeeping,
    Metabolic,
    Binding,
    Nuclease,
    Polymerase,
    ReverseTranscriptase,
    ProgrammableNucleaseSystem,
    Unknown,
}

impl IntendedFunction {
    pub fn forces_screen(self) -> bool {
        matches!(
            self,
            Self::Nuclease
                | Self::Polymerase
                | Self::ReverseTranscriptase
                | Self::ProgrammableNucleaseSystem
                | Self::Unknown
        )
    }

    pub fn escalates_if_uncleared(self) -> bool {
        matches!(
            self,
            Self::ReverseTranscriptase | Self::ProgrammableNucleaseSystem | Self::Unknown
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bsl {
    #[serde(rename = "BSL-1")]
    Bsl1,
    #[serde(rename = "BSL-2")]
    Bsl2,
    #[serde(rename = "BSL-3")]
    Bsl3,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fragment {
    pub id: String,
    pub role: Role,
    pub alphabet: Alphabet,
    pub sequence: String,
    #[serde(default)]
    pub coords: Option<Coords>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Cds,
    Promoter,
    Rbs,
    Terminator,
    RepeatArray,
    Spacer,
    Oligo,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alphabet {
    Dna,
    Rna,
    Aa,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coords {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assembly {
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(default)]
    pub order_split: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum CensorError {
    #[error("spec_version must be 0.1.0")]
    Version,
    #[error("BSL-3 is out of scope")]
    Bsl3,
    #[error("order_split is forbidden")]
    OrderSplit,
    #[error("no fragments")]
    Empty,
    #[error("stitched DNA exceeds 50 kb v0 cap")]
    TooLong,
    #[error("fragment {0}: alphabet/sequence mismatch")]
    Alphabet(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

impl DesignSpec {
    pub fn from_json(bytes: &[u8]) -> Result<Self, CensorError> {
        let spec: Self = serde_json::from_slice(bytes)?;
        spec.censor()?;
        Ok(spec)
    }

    pub fn censor(&self) -> Result<(), CensorError> {
        if self.spec_version != "0.1.0" {
            return Err(CensorError::Version);
        }
        if self.intended_bsl == Bsl::Bsl3 {
            return Err(CensorError::Bsl3);
        }
        if self
            .assembly
            .as_ref()
            .and_then(|a| a.order_split)
            .unwrap_or(false)
        {
            return Err(CensorError::OrderSplit);
        }
        if self.fragments.is_empty() {
            return Err(CensorError::Empty);
        }
        let mut dna_len = 0usize;
        for f in &self.fragments {
            if !alphabet_ok(f.alphabet, &f.sequence) {
                return Err(CensorError::Alphabet(f.id.clone()));
            }
            if matches!(f.alphabet, Alphabet::Dna | Alphabet::Rna) {
                dna_len += f.sequence.chars().filter(|c| !c.is_whitespace()).count();
            }
        }
        if dna_len > 50_000 {
            return Err(CensorError::TooLong);
        }
        Ok(())
    }
}

fn alphabet_ok(alphabet: Alphabet, seq: &str) -> bool {
    let s = seq.chars().filter(|c| !c.is_whitespace());
    match alphabet {
        Alphabet::Dna => s.clone().all(|c| "ACGTacgtN*-".contains(c)),
        Alphabet::Rna => s.clone().all(|c| "ACGUacguN*-".contains(c)),
        Alphabet::Aa => s.clone().all(|c| "ACDEFGHIKLMNPQRSTVWYacdefghiklmnpqrstvwyXx*-".contains(c)),
    }
}

#[cfg(test)]
mod tests {
    use super::DesignSpec;

    #[test]
    fn gfp_fixture_passes_censor() {
        let bytes = include_bytes!("../fixtures/benign/gfp_spec.json");
        DesignSpec::from_json(bytes).unwrap();
    }

    #[test]
    fn extra_key_is_refused() {
        let raw = br#"{"spec_version":"0.1.0","construct_id":"x","designer":{"agent":"a","human":"h"},"chassis":"none","intended_function":"reporter","intended_bsl":"BSL-1","not_for_synthesis":true,"fragments":[{"id":"f","role":"cds","alphabet":"dna","sequence":"ATGC"}],"justification":"no"}"#;
        assert!(DesignSpec::from_json(raw).is_err());
    }

    #[test]
    fn bsl3_is_refused() {
        let raw = br#"{"spec_version":"0.1.0","construct_id":"x","designer":{"agent":"a","human":"h"},"chassis":"none","intended_function":"reporter","intended_bsl":"BSL-3","not_for_synthesis":true,"fragments":[{"id":"f","role":"cds","alphabet":"dna","sequence":"ATGC"}]}"#;
        assert!(DesignSpec::from_json(raw).is_err());
    }
}

