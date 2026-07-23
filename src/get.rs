//! `loam get` — retrieve a concept with read-time anchor verification
//! (Consumption §4.2). Trust tier always surfaced; STALE flagged inline when an
//! anchor no longer matches source; a `concept_read` event is spooled.

use crate::concept::parse_concept;
use crate::spool::{Event, Spool};
use anyhow::{Context, Result};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GetResult {
    pub concept_id: String,
    pub trust_tier: String,
    pub body: String,
    pub stale: bool,
    /// repo-relative paths of anchors whose source no longer matches.
    pub changed_anchors: Vec<String>,
}

/// Read a concept from `bundle_dir`, verify its anchors against the working
/// tree (source root = the bundle's parent), and (best-effort) spool a
/// `concept_read` event. `spool` is optional so the operation is testable
/// without telemetry.
pub fn get(
    bundle_dir: &Path,
    concept_id: &str,
    spool: Option<&Spool>,
    harness: &str,
    task: &str,
) -> Result<GetResult> {
    let path = bundle_dir.join(format!("{concept_id}.md"));
    let concept =
        parse_concept(&path).with_context(|| format!("get: concept '{concept_id}'"))?;
    let source_root = bundle_dir.parent().unwrap_or(bundle_dir);

    let mut changed_anchors = Vec::new();
    for a in &concept.frontmatter.sources {
        if !a.is_fresh(source_root) {
            changed_anchors.push(a.path.clone());
        }
    }
    let stale = !changed_anchors.is_empty();

    if let Some(s) = spool {
        let payload = serde_json::json!({
            "trust_tier": concept.frontmatter.trust_tier,
            "stale": stale,
        })
        .to_string();
        s.emit_best_effort(&Event::now(
            "concept_read",
            harness,
            task,
            &bundle_dir.to_string_lossy(),
            payload,
        ));
    }

    Ok(GetResult {
        concept_id: concept.frontmatter.concept_id,
        trust_tier: concept.frontmatter.trust_tier,
        body: concept.body,
        stale,
        changed_anchors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn bundle() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_repo/loam")
    }

    #[test]
    fn fresh_concept_not_stale_with_tier() {
        let r = get(&bundle(), "greeting-contract", None, "test", "t").unwrap();
        assert_eq!(r.trust_tier, "verified");
        assert!(!r.stale);
        assert!(r.changed_anchors.is_empty());
        assert!(r.body.contains("greet"));
    }

    #[test]
    fn stale_concept_flags_changed_anchor() {
        let r = get(&bundle(), "stale-example", None, "test", "t").unwrap();
        assert!(r.stale);
        assert_eq!(r.changed_anchors, vec!["src/greeting.rs".to_string()]);
    }

    #[test]
    fn emits_concept_read_event() {
        let dir = tempfile::tempdir().unwrap();
        let spool = Spool::open(&dir.path().join("spool.sqlite")).unwrap();
        get(&bundle(), "greeting-contract", Some(&spool), "claude-code", "t").unwrap();
        assert_eq!(spool.count_kind("concept_read").unwrap(), 1);
    }
}
