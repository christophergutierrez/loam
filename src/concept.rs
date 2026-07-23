//! OKF concept model: frontmatter (Supply §4, canonical typed anchor
//! path+content_hash+span+quote per ADR-0006 review), body, and read-time
//! anchor verification against the working tree (Consumption §4.2).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// A typed evidence anchor. No anchor, no admission (Supply §7.1).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Anchor {
    pub path: String,
    pub content_hash: String,
    pub span: [usize; 2],
    #[serde(default)]
    pub quote: String,
}

impl Anchor {
    /// True iff the source span still hashes to the recorded content_hash.
    /// A missing source file counts as stale (not fresh).
    pub fn is_fresh(&self, source_root: &Path) -> bool {
        match std::fs::read_to_string(source_root.join(&self.path)) {
            Ok(text) => hash_span(&text, self.span[0], self.span[1]) == self.content_hash,
            Err(_) => false,
        }
    }
}

/// sha256 (hex) of the inclusive 1-based line span. Canonical form (ADR-0008):
/// CRLF/CR are normalized to LF before hashing, so the Python supply side
/// (writer) and the Rust demand side (verifier) agree regardless of the source
/// file's on-disk line endings. Lines are joined with '\n'.
pub fn hash_span(text: &str, start: usize, end: usize) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();
    let s = start.saturating_sub(1).min(lines.len());
    let e = end.min(lines.len());
    let span = if s <= e {
        lines[s..e].join("\n")
    } else {
        String::new()
    };
    let mut h = Sha256::new();
    h.update(span.as_bytes());
    format!("{:x}", h.finalize())
}

/// A concept id must be a flat slug — no path separators or `..` traversal — so
/// a crafted id (`loam get ../../x`) or a crafted link (`](../evil.md)`) cannot
/// escape the bundle directory and serve an out-of-bundle file as in-bundle.
pub fn is_valid_concept_id(id: &str) -> bool {
    !id.is_empty() && !id.contains('/') && !id.contains('\\') && !id.contains("..")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Frontmatter {
    pub concept_id: String,
    pub trust_tier: String,
    #[serde(default)]
    pub claim_type: String,
    #[serde(default)]
    pub sample_stream: String,
    #[serde(default)]
    pub sources: Vec<Anchor>,
}

#[derive(Debug, Clone)]
pub struct Concept {
    pub frontmatter: Frontmatter,
    pub body: String,
    pub path: PathBuf,
}

/// Parse an OKF concept markdown file (leading `---` YAML frontmatter + body).
pub fn parse_concept(path: &Path) -> Result<Concept> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading concept {}", path.display()))?;
    let (fm, body) = split_frontmatter(&text)
        .with_context(|| format!("parsing frontmatter of {}", path.display()))?;
    let frontmatter: Frontmatter = serde_yaml::from_str(fm)
        .with_context(|| format!("frontmatter yaml of {}", path.display()))?;
    Ok(Concept {
        frontmatter,
        body: body.trim_start().to_string(),
        path: path.to_path_buf(),
    })
}

fn split_frontmatter(text: &str) -> Result<(&str, &str)> {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
        .ok_or_else(|| anyhow::anyhow!("missing leading '---' frontmatter fence"))?;
    if let Some(i) = rest.find("\n---\n") {
        Ok((&rest[..i], &rest[i + 5..]))
    } else if let Some(i) = rest.find("\n---") {
        Ok((&rest[..i], &rest[i + 4..]))
    } else {
        bail!("unterminated frontmatter (no closing '---')")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_repo/loam")
    }

    #[test]
    fn parses_concept_frontmatter_and_anchor() {
        let c = parse_concept(&bundle().join("greeting-contract.md")).unwrap();
        assert_eq!(c.frontmatter.concept_id, "greeting-contract");
        assert_eq!(c.frontmatter.trust_tier, "verified");
        assert_eq!(c.frontmatter.sources.len(), 1);
        assert_eq!(c.frontmatter.sources[0].path, "src/greeting.rs");
        assert!(c.body.contains("greet"));
    }

    #[test]
    fn fresh_anchor_verifies_against_source() {
        let c = parse_concept(&bundle().join("greeting-contract.md")).unwrap();
        let source_root = bundle().parent().unwrap().to_path_buf();
        assert!(c.frontmatter.sources[0].is_fresh(&source_root));
    }

    #[test]
    fn wrong_hash_anchor_is_stale() {
        let c = parse_concept(&bundle().join("stale-example.md")).unwrap();
        let source_root = bundle().parent().unwrap().to_path_buf();
        assert!(!c.frontmatter.sources[0].is_fresh(&source_root));
    }

    #[test]
    fn hash_is_line_ending_invariant() {
        // CRLF and LF versions of the same span must hash identically (ADR-0008).
        assert_eq!(
            hash_span("a\r\nb\r\nc", 1, 2),
            hash_span("a\nb\nc", 1, 2),
            "CRLF must normalize to LF before hashing"
        );
    }

    #[test]
    fn concept_id_validation_rejects_traversal() {
        assert!(is_valid_concept_id("greeting-contract"));
        assert!(is_valid_concept_id("_index"));
        assert!(!is_valid_concept_id("../evil"));
        assert!(!is_valid_concept_id("a/b"));
        assert!(!is_valid_concept_id("a\\b"));
        assert!(!is_valid_concept_id(""));
    }
}
