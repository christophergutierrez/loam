//! `loam observe` — file a typed observation into the inbox (Consumption §6).
//! The write boundary: consuming agents write ONLY the inbox, never concepts
//! (invariant: agents-never-write). Observations are untrusted hints the
//! pipeline re-derives from source; nothing here is admitted as knowledge.

use crate::spool::{Event, Spool};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// The typed observation kinds accepted by the inbox.
pub const KINDS: [&str; 5] = [
    "claim",
    "contradiction",
    "concept-wrong",
    "concept-missing",
    "procedural",
];

/// Write a typed observation as a JSON file into `inbox_dir`. Returns the path
/// written. Rejects unknown kinds. Emits `observation_filed`. Never touches the
/// concept bundle.
pub fn observe(
    inbox_dir: &Path,
    kind: &str,
    body: &str,
    spool: Option<&Spool>,
    harness: &str,
    task: &str,
) -> Result<PathBuf> {
    if !KINDS.contains(&kind) {
        bail!("unknown observation kind '{kind}'; expected one of {KINDS:?}");
    }
    std::fs::create_dir_all(inbox_dir)?;

    let ts_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let entry = serde_json::json!({
        "kind": kind,
        "body": body,
        "harness": harness,
        "task": task,
        "ts_ms": ts_ms,
        // untrusted: the pipeline re-derives evidence from source; this is a hint.
        "trusted": false,
    });
    let bytes = serde_json::to_vec_pretty(&entry)?;

    // Write to a fresh filename via create_new so a surviving entry is NEVER
    // overwritten — even after the pipeline has drained some entries (the
    // count-based scheme collided with survivors and fs::write clobbered them).
    // ts_ms orders entries; the suffix disambiguates same-millisecond writes.
    use std::io::Write;
    let mut suffix = 0u32;
    let path = loop {
        let candidate = inbox_dir.join(format!("{ts_ms:013}-{kind}-{suffix}.json"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut f) => {
                f.write_all(&bytes)?;
                break candidate;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => suffix += 1,
            Err(e) => return Err(e.into()),
        }
    };

    if let Some(s) = spool {
        let payload = serde_json::json!({ "kind": kind }).to_string();
        s.emit_best_effort(&Event::now(
            "observation_filed",
            harness,
            task,
            &inbox_dir.to_string_lossy(),
            payload,
        ));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_typed_inbox_entry_as_json() {
        let dir = tempfile::tempdir().unwrap();
        let inbox = dir.path().join("inbox");
        let p = observe(
            &inbox,
            "contradiction",
            "doc says X, code does Y",
            None,
            "test",
            "t",
        )
        .unwrap();
        assert!(p.starts_with(&inbox), "entry must be under inbox: {p:?}");
        assert_eq!(p.extension().and_then(|e| e.to_str()), Some("json"));
        let content = std::fs::read_to_string(&p).unwrap();
        assert!(content.contains("contradiction"));
        assert!(content.contains("doc says X"));
    }

    #[test]
    fn rejects_unknown_kind() {
        let dir = tempfile::tempdir().unwrap();
        let r = observe(&dir.path().join("inbox"), "bogus", "x", None, "test", "t");
        assert!(r.is_err(), "unknown kind must be rejected");
    }

    #[test]
    fn emits_observation_filed_and_writes_no_concept() {
        let dir = tempfile::tempdir().unwrap();
        let inbox = dir.path().join("inbox");
        let spool = Spool::open(&dir.path().join("spool.sqlite")).unwrap();
        observe(
            &inbox,
            "claim",
            "greet also trims whitespace",
            Some(&spool),
            "hermes",
            "t",
        )
        .unwrap();
        assert_eq!(spool.count_kind("observation_filed").unwrap(), 1);
        // never writes a concept: only .json under inbox, never a .md
        for e in std::fs::read_dir(&inbox).unwrap() {
            let p = e.unwrap().path();
            assert_ne!(p.extension().and_then(|x| x.to_str()), Some("md"));
        }
    }

    #[test]
    fn survives_partial_drain_without_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let inbox = dir.path().join("inbox");
        let a = observe(&inbox, "claim", "aaa", None, "t", "t").unwrap();
        let b = observe(&inbox, "claim", "bbb", None, "t", "t").unwrap();
        assert_ne!(a, b);
        // pipeline drains 'a'
        std::fs::remove_file(&a).unwrap();
        // a new observation must not clobber the surviving 'b'
        let c = observe(&inbox, "claim", "ccc", None, "t", "t").unwrap();
        assert_ne!(b, c);
        assert!(
            std::fs::read_to_string(&b).unwrap().contains("bbb"),
            "surviving entry b must be intact, not overwritten"
        );
        assert!(std::fs::read_to_string(&c).unwrap().contains("ccc"));
    }
}
