use bloc::commec;
use bloc::hash::{fasta_hash, identity_key, spec_hash};
use bloc::pattern;
use bloc::reduce::{self, EngineSignals};
use bloc::seq::{stitch_fasta, stitched_sequence};
use bloc::spec::DesignSpec;
use bloc::verdict::{Receipt, Verdict};
use bloc::window;
use bloc::VERSION;
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "bloc", version = VERSION, about = "Censor + suppressor frame for designer sequences")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Speccheck { spec: PathBuf },
    Screen {
        spec: PathBuf,
        #[arg(long)]
        commec_db: Option<PathBuf>,
        #[arg(long)]
        commec_bin: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Card { verdict: PathBuf },
    Receipt { verdict: PathBuf },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Speccheck { spec } => match load_spec(&spec) {
            Ok(s) => {
                println!("OK {}", s.construct_id);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("CENSOR {e}");
                ExitCode::from(2)
            }
        },
        Cmd::Screen {
            spec,
            commec_db,
            commec_bin,
            out,
        } => match screen(&spec, commec_db.as_deref(), commec_bin.as_deref(), out.as_deref()) {
            Ok(v) => {
                println!("{}", bloc::card::render(&v));
                ExitCode::from(v.policy.exit_code() as u8)
            }
            Err(e) => {
                eprintln!("CENSOR {e}");
                ExitCode::from(2)
            }
        },
        Cmd::Card { verdict } => match load_verdict(&verdict) {
            Ok(v) => {
                println!("{}", bloc::card::render(&v));
                ExitCode::from(v.policy.exit_code() as u8)
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::from(2)
            }
        },
        Cmd::Receipt { verdict } => match load_verdict(&verdict) {
            Ok(v) => {
                println!("{}", serde_json::to_string_pretty(&v.receipt).unwrap());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::from(2)
            }
        },
    }
}

fn load_spec(path: &Path) -> Result<DesignSpec, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    DesignSpec::from_json(&bytes).map_err(|e| e.to_string())
}

fn load_verdict(path: &Path) -> Result<Verdict, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn screen(
    spec_path: &Path,
    commec_db: Option<&Path>,
    commec_bin: Option<&str>,
    out: Option<&Path>,
) -> Result<Verdict, String> {
    let bytes = fs::read(spec_path).map_err(|e| e.to_string())?;
    let spec = DesignSpec::from_json(&bytes).map_err(|e| e.to_string())?;
    let spec_val: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let fasta = stitch_fasta(&spec);
    let seq = stitched_sequence(&spec);
    let spec_sha = spec_hash(&spec_val);
    let fasta_sha = fasta_hash(&fasta);
    let id = identity_key(&spec_sha, &fasta_sha);

    let tmp = std::env::temp_dir().join(format!("{}.stitched.fa", spec.construct_id));
    fs::write(&tmp, &fasta).map_err(|e| e.to_string())?;
    let commec_out = commec::run(commec_db, commec_bin, &tmp);
    let _ = fs::remove_file(&tmp);

    let pat = pattern::detect(&spec, &seq);
    let commec_required = true;
    let (rubric, policy, reasons) = reduce::reduce(&EngineSignals {
        commec_required,
        commec_ran: commec_out.ran,
        commec_flag: commec_out.flag,
        pattern: pat.clone(),
        intended: spec.intended_function,
    });

    let verdict = Verdict {
        verdict_version: "0.1.0".into(),
        construct_id: spec.construct_id.clone(),
        rubric,
        policy,
        reasons,
        engines: json!({
            "commec": commec_out.raw,
            "windows": window::stats(&seq),
            "pattern": pat
        }),
        receipt: Receipt {
            spec_sha256: spec_sha,
            fasta_sha256: fasta_sha,
            identity: id,
            inhibit_version: VERSION.into(),
            engine_revs: json!({
                "bloc": VERSION,
                "commec": if commec_out.ran { "ran" } else { "absent" }
            }),
            utc: chrono::Utc::now().to_rfc3339(),
        },
    };

    let dest = out
        .map(PathBuf::from)
        .unwrap_or_else(|| spec_path.with_extension("verdict.json"));
    fs::write(&dest, serde_json::to_vec_pretty(&verdict).unwrap()).map_err(|e| e.to_string())?;
    Ok(verdict)
}
