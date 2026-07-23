//! `loam search` — text/frontmatter search over the bundle. A zero-result
//! search emits a `search_miss` event (Consumption §4: the demand sensor).

use crate::concept::parse_concept;
use crate::spool::{Event, Spool};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub concept_id: String,
    pub trust_tier: String,
}

/// Search the (flat) bundle for concepts matching all whitespace-separated
/// terms in `query` (case-insensitive, over concept_id + body). Emits a
/// `search_miss` event when nothing matches.
// TODO(anchor-index): this full-scans and parses every concept on each call —
// fine at fixture scale, wrong at census scale. The derived anchor index
// (Supply §4) is the spec'd successor. This is placeholder-with-a-known-
// successor, not a design decision.
pub fn search(
    bundle_dir: &Path,
    query: &str,
    spool: Option<&Spool>,
    harness: &str,
    task: &str,
) -> Result<Vec<SearchHit>> {
    let q = query.to_lowercase();
    let terms: Vec<&str> = q.split_whitespace().collect();
    let mut hits = Vec::new();

    for entry in std::fs::read_dir(bundle_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let concept = match parse_concept(&path) {
            Ok(c) => c,
            Err(_) => continue, // not a well-formed concept; skip
        };
        let hay = format!("{} {}", concept.frontmatter.concept_id, concept.body).to_lowercase();
        if !terms.is_empty() && terms.iter().all(|t| hay.contains(t)) {
            hits.push(SearchHit {
                concept_id: concept.frontmatter.concept_id,
                trust_tier: concept.frontmatter.trust_tier,
            });
        }
    }
    hits.sort_by(|a, b| a.concept_id.cmp(&b.concept_id));

    if hits.is_empty() {
        if let Some(s) = spool {
            let payload = serde_json::json!({ "query": query }).to_string();
            s.emit_best_effort(&Event::now(
                "search_miss",
                harness,
                task,
                &bundle_dir.to_string_lossy(),
                payload,
            ));
        }
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn bundle() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_repo/loam")
    }

    #[test]
    fn finds_concept_by_term() {
        let hits = search(&bundle(), "greet", None, "test", "t").unwrap();
        assert!(
            hits.iter().any(|h| h.concept_id == "greeting-contract"),
            "expected greeting-contract in {hits:?}"
        );
    }

    #[test]
    fn zero_result_emits_search_miss() {
        let dir = tempfile::tempdir().unwrap();
        let spool = Spool::open(&dir.path().join("spool.sqlite")).unwrap();
        let hits = search(&bundle(), "zzzznotarealtermzzzz", Some(&spool), "test", "t").unwrap();
        assert!(hits.is_empty());
        assert_eq!(spool.count_kind("search_miss").unwrap(), 1);
    }
}
