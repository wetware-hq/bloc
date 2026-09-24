use crate::spec::{DesignSpec, IntendedFunction, Role};

#[derive(Debug, Clone, serde::Serialize)]
pub struct PatternHit {
    pub repeat_array: bool,
    pub rt_like_cds: bool,
    pub programmable_system_shape: bool,
}

/// Structural ART-shaped neighborhood. No pathogen table.
pub fn detect(spec: &DesignSpec, stitched: &str) -> PatternHit {
    let declared_array = spec.fragments.iter().any(|f| f.role == Role::RepeatArray);
    let repeat_array = declared_array || even_repeats(stitched);
    let rt_like_cds = spec.intended_function == IntendedFunction::ReverseTranscriptase
        || spec.fragments.iter().any(|f| {
            f.role == Role::Cds && normalize_len(f.sequence.len()) >= 900 && normalize_len(f.sequence.len()) <= 2500
        });
    let programmable = spec.intended_function == IntendedFunction::ProgrammableNucleaseSystem
        || (repeat_array && rt_like_cds);
    PatternHit {
        repeat_array,
        rt_like_cds,
        programmable_system_shape: programmable,
    }
}

fn normalize_len(n: usize) -> usize {
    n
}

fn even_repeats(seq: &str) -> bool {
    // ≥6 units, period 20–50, same motif.
    if seq.len() < 120 {
        return false;
    }
    for period in 20..=50 {
        let unit = period;
        if seq.len() < unit * 6 {
            continue;
        }
        let motif = &seq[..unit];
        if motif.chars().all(|c| c == 'N' || c == 'A' || c == 'T' || c == 'G' || c == 'C') {
            let mut hits = 0usize;
            let mut i = 0usize;
            while i + unit <= seq.len() {
                if &seq[i..i + unit] == motif {
                    hits += 1;
                    i += unit;
                } else {
                    break;
                }
            }
            if hits >= 6 {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::even_repeats;

    #[test]
    fn synthetic_array_fires() {
        let unit = "AAAAAAAAAAAAAAAAAAAA";
        let seq = unit.repeat(8);
        assert!(even_repeats(&seq));
    }

    #[test]
    fn random_short_silent() {
        assert!(!even_repeats("ATGAAACCCGGGTTT"));
    }
}
