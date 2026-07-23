//! `loam bundle <task>` — dumb-first task-scoped context assembly
//! (Consumption §5.3): seed by search, then follow explicit markdown links,
//! size-capped, trust tiers surfaced. Emits `bundle_assembled`.

use crate::concept::parse_concept;
use crate::search::search;
use crate::spool::{Event, Spool};
use anyhow::Result;
use std::collections::{BTreeSet, VecDeque};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BundleEntry {
    pub concept_id: String,
    pub trust_tier: String,
}

#[derive(Debug, Clone)]
pub struct AssembledBundle {
    pub concepts: Vec<BundleEntry>,
    pub capped: bool,
}

/// Extract in-bundle markdown link targets (`](name.md)`), ignoring URLs.
pub fn extract_links(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(i) = rest.find("](") {
        let after = &rest[i + 2..];
        if let Some(j) = after.find(')') {
            let target = &after[..j];
            if target.ends_with(".md") && !target.contains("://") {
                out.push(target.to_string());
            }
            rest = &after[j + 1..];
        } else {
            break;
        }
    }
    out
}

pub fn assemble(
    bundle_dir: &Path,
    task: &str,
    max_concepts: usize,
    spool: Option<&Spool>,
    harness: &str,
    task_id: &str,
) -> Result<AssembledBundle> {
    // Seed with search hits; fall back to the bundle-root index concept.
    let seeds = search(bundle_dir, task, None, harness, task_id)?;
    let mut queue: VecDeque<String> = seeds.into_iter().map(|h| h.concept_id).collect();
    if queue.is_empty() {
        queue.push_back("_index".to_string());
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut concepts: Vec<BundleEntry> = Vec::new();
    let mut capped = false;

    while let Some(id) = queue.pop_front() {
        if seen.contains(&id) {
            continue;
        }
        if concepts.len() >= max_concepts {
            capped = true;
            break;
        }
        let path = bundle_dir.join(format!("{id}.md"));
        let concept = match parse_concept(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        seen.insert(id.clone());
        concepts.push(BundleEntry {
            concept_id: concept.frontmatter.concept_id.clone(),
            trust_tier: concept.frontmatter.trust_tier.clone(),
        });
        for link in extract_links(&concept.body) {
            let lid = link.trim_end_matches(".md").to_string();
            if !seen.contains(&lid) {
                queue.push_back(lid);
            }
        }
    }

    if let Some(s) = spool {
        let payload = serde_json::json!({
            "task": task,
            "concepts": concepts.iter().map(|c| &c.concept_id).collect::<Vec<_>>(),
            "capped": capped,
        })
        .to_string();
        s.emit_best_effort(&Event::now(
            "bundle_assembled",
            harness,
            task_id,
            &bundle_dir.to_string_lossy(),
            payload,
        ));
    }

    Ok(AssembledBundle { concepts, capped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn bundle() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_repo/loam")
    }

    fn ids(b: &AssembledBundle) -> Vec<String> {
        b.concepts.iter().map(|c| c.concept_id.clone()).collect()
    }

    #[test]
    fn assembles_seed_and_linked_concepts() {
        let b = assemble(&bundle(), "greet", 16, None, "test", "t").unwrap();
        let got = ids(&b);
        assert!(
            got.contains(&"greeting-contract".to_string()),
            "seed missing: {got:?}"
        );
        assert!(
            got.contains(&"_index".to_string()),
            "linked _index missing: {got:?}"
        );
    }

    #[test]
    fn respects_size_cap() {
        let b = assemble(&bundle(), "greet", 1, None, "test", "t").unwrap();
        assert_eq!(b.concepts.len(), 1);
        assert!(b.capped);
    }

    #[test]
    fn emits_bundle_assembled() {
        let dir = tempfile::tempdir().unwrap();
        let spool = Spool::open(&dir.path().join("spool.sqlite")).unwrap();
        assemble(&bundle(), "greet", 16, Some(&spool), "test", "t").unwrap();
        assert_eq!(spool.count_kind("bundle_assembled").unwrap(), 1);
    }
}
