//! `loam` — thin CLI shell over `loam_core` (ADR-0006). The library holds the
//! logic; this binary parses args, resolves the bundle, and prints.

use clap::{Parser, Subcommand};
use loam_core::assemble::{assemble, AssembledBundle};
use loam_core::bundle::resolve_bundle;
use loam_core::get::{get, GetResult};
use loam_core::observe::observe;
use loam_core::search::{search, SearchHit};
use loam_core::spool::Spool;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "loam",
    version,
    about = "Loam demand-side CLI (loam-core, ADR-0006)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
    /// Emit machine-readable JSON.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Cmd {
    /// Retrieve a concept with read-time anchor verification.
    Get { concept: String },
    /// Find concepts matching the given terms.
    Search { terms: Vec<String> },
    /// Assemble a task-scoped context bundle (seed by search + link traversal).
    Bundle {
        task: Vec<String>,
        /// Max concepts to include (size cap).
        #[arg(long, default_value_t = 16)]
        max: usize,
    },
    /// File a typed observation into the inbox (never writes a concept).
    Observe {
        /// One of: claim, contradiction, concept-wrong, concept-missing, procedural.
        kind: String,
        text: Vec<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let bundle_dir = match resolve_bundle(&cwd) {
        Some(b) => b,
        None => {
            eprintln!(
                "loam: no OKF bundle found (no loam/_index.md above {})",
                cwd.display()
            );
            return ExitCode::from(3);
        }
    };
    let spool = open_spool(&bundle_dir);
    let harness = std::env::var("LOAM_HARNESS").unwrap_or_else(|_| "unknown".into());

    match cli.cmd {
        Cmd::Get { concept } => match get(&bundle_dir, &concept, spool.as_ref(), &harness, "cli") {
            Ok(r) => {
                print_get(&r, cli.json);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("loam get: {e:#}");
                ExitCode::from(1)
            }
        },
        Cmd::Search { terms } => {
            let query = terms.join(" ");
            match search(&bundle_dir, &query, spool.as_ref(), &harness, "cli") {
                Ok(hits) => {
                    print_search(&hits, cli.json);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("loam search: {e:#}");
                    ExitCode::from(1)
                }
            }
        }
        Cmd::Bundle { task, max } => {
            let task = task.join(" ");
            match assemble(&bundle_dir, &task, max, spool.as_ref(), &harness, "cli") {
                Ok(b) => {
                    print_bundle(&b, cli.json);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("loam bundle: {e:#}");
                    ExitCode::from(1)
                }
            }
        }
        Cmd::Observe { kind, text } => {
            let inbox = inbox_dir(&bundle_dir);
            let body = text.join(" ");
            match observe(&inbox, &kind, &body, spool.as_ref(), &harness, "cli") {
                Ok(p) => {
                    if cli.json {
                        println!("{}", serde_json::json!({"filed": p.to_string_lossy()}));
                    } else {
                        println!("observation filed: {}", p.display());
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("loam observe: {e:#}");
                    ExitCode::from(1)
                }
            }
        }
    }
}

/// Inbox location: `LOAM_INBOX` override, else `<repo>/.loam/inbox`.
fn inbox_dir(bundle_dir: &Path) -> PathBuf {
    std::env::var_os("LOAM_INBOX")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            bundle_dir
                .parent()
                .unwrap_or(bundle_dir)
                .join(".loam/inbox")
        })
}

/// Open the local spool. Path override via `LOAM_SPOOL`, else the bundle's
/// rebuildable `.loam-cache/`. Failure is non-fatal (spool-only, best-effort).
fn open_spool(bundle_dir: &Path) -> Option<Spool> {
    let path = std::env::var_os("LOAM_SPOOL")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            bundle_dir
                .parent()
                .unwrap_or(bundle_dir)
                .join(".loam-cache/spool.sqlite")
        });
    match Spool::open(&path) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("loam: telemetry spool unavailable ({e}); continuing");
            None
        }
    }
}

fn print_search(hits: &[SearchHit], json: bool) {
    if json {
        let v: Vec<_> = hits
            .iter()
            .map(|h| serde_json::json!({"concept_id": h.concept_id, "trust_tier": h.trust_tier}))
            .collect();
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else if hits.is_empty() {
        println!("no matches");
    } else {
        for h in hits {
            println!("[{}] {}", h.trust_tier, h.concept_id);
        }
    }
}

fn print_bundle(b: &AssembledBundle, json: bool) {
    if json {
        let v = serde_json::json!({
            "concepts": b.concepts.iter()
                .map(|c| serde_json::json!({"concept_id": c.concept_id, "trust_tier": c.trust_tier}))
                .collect::<Vec<_>>(),
            "capped": b.capped,
        });
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        for c in &b.concepts {
            println!("[{}] {}", c.trust_tier, c.concept_id);
        }
        if b.capped {
            println!("… (size cap reached; more concepts available)");
        }
    }
}

fn print_get(r: &GetResult, json: bool) {
    if json {
        let v = serde_json::json!({
            "concept_id": r.concept_id,
            "trust_tier": r.trust_tier,
            "stale": r.stale,
            "changed_anchors": r.changed_anchors,
            "body": r.body,
        });
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        println!("[{}] {}", r.trust_tier, r.concept_id);
        if r.stale {
            println!(
                "⚠ STALE — anchors no longer match source: {}",
                r.changed_anchors.join(", ")
            );
            println!("  (verify against source before relying on this)");
        }
        println!("\n{}", r.body);
    }
}
