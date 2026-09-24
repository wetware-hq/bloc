pub const WINDOW_NT: usize = 50;
pub const OLIGO_FLOOR_NT: usize = 30;

pub fn windows(seq: &str, size: usize) -> Vec<String> {
    if seq.len() < size {
        return if seq.is_empty() {
            vec![]
        } else {
            vec![seq.to_string()]
        };
    }
    seq.as_bytes()
        .windows(size)
        .map(|w| String::from_utf8_lossy(w).into_owned())
        .collect()
}

pub fn six_frame_aa(seq: &str) -> Vec<String> {
    let table = |codon: &[u8]| -> char {
        match codon {
            b"TTT" | b"TTC" => 'F',
            b"TTA" | b"TTG" | b"CTT" | b"CTC" | b"CTA" | b"CTG" => 'L',
            b"ATT" | b"ATC" | b"ATA" => 'I',
            b"ATG" => 'M',
            b"GTT" | b"GTC" | b"GTA" | b"GTG" => 'V',
            b"TCT" | b"TCC" | b"TCA" | b"TCG" | b"AGT" | b"AGC" => 'S',
            b"CCT" | b"CCC" | b"CCA" | b"CCG" => 'P',
            b"ACT" | b"ACC" | b"ACA" | b"ACG" => 'T',
            b"GCT" | b"GCC" | b"GCA" | b"GCG" => 'A',
            b"TAT" | b"TAC" => 'Y',
            b"CAT" | b"CAC" => 'H',
            b"CAA" | b"CAG" => 'Q',
            b"AAT" | b"AAC" => 'N',
            b"AAA" | b"AAG" => 'K',
            b"GAT" | b"GAC" => 'D',
            b"GAA" | b"GAG" => 'E',
            b"TGT" | b"TGC" => 'C',
            b"TGG" => 'W',
            b"CGT" | b"CGC" | b"CGA" | b"CGG" | b"AGA" | b"AGG" => 'R',
            b"GGT" | b"GGC" | b"GGA" | b"GGG" => 'G',
            b"TAA" | b"TAG" | b"TGA" => '*',
            _ => 'X',
        }
    };
    let bytes = seq.as_bytes();
    let mut out = Vec::new();
    for frame in 0..3 {
        let mut aa = String::new();
        let mut i = frame;
        while i + 3 <= bytes.len() {
            aa.push(table(&bytes[i..i + 3]));
            i += 3;
        }
        if !aa.is_empty() {
            out.push(aa);
        }
    }
    out
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WindowStats {
    pub n50: usize,
    pub n30: usize,
    pub seq_len: usize,
}

pub fn stats(seq: &str) -> WindowStats {
    WindowStats {
        n50: windows(seq, WINDOW_NT).len(),
        n30: windows(seq, OLIGO_FLOOR_NT).len(),
        seq_len: seq.len(),
    }
}
