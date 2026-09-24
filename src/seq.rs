use crate::spec::{Alphabet, DesignSpec};

pub fn normalize_nt(alphabet: Alphabet, raw: &str) -> String {
    raw.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| {
            let u = c.to_ascii_uppercase();
            if alphabet == Alphabet::Rna && u == 'U' {
                'T'
            } else {
                u
            }
        })
        .collect()
}

pub fn stitch_fasta(spec: &DesignSpec) -> String {
    let mut seq = String::new();
    for f in &spec.fragments {
        match f.alphabet {
            Alphabet::Dna | Alphabet::Rna => seq.push_str(&normalize_nt(f.alphabet, &f.sequence)),
            Alphabet::Aa => {}
        }
    }
    format!(">{}\n{}\n", spec.construct_id, seq)
}

pub fn stitched_sequence(spec: &DesignSpec) -> String {
    spec.fragments
        .iter()
        .filter(|f| matches!(f.alphabet, Alphabet::Dna | Alphabet::Rna))
        .map(|f| normalize_nt(f.alphabet, &f.sequence))
        .collect()
}
