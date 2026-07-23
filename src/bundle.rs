//! Corpus resolution and (later) OKF concept model.

use std::path::{Path, PathBuf};

/// Walk up from `start` to the nearest-ancestor OKF bundle root
/// (Consumption §4.1). A bundle root is a directory containing
/// `loam/_index.md` — the bundle-root index concept. Returns the path to the
/// `loam/` bundle directory, or `None` if `start` is not inside a bundle.
pub fn resolve_bundle(start: &Path) -> Option<PathBuf> {
    // Start at `start` if it's a directory, else its parent.
    let mut dir: Option<&Path> = if start.is_dir() {
        Some(start)
    } else {
        start.parent()
    };
    while let Some(d) = dir {
        let candidate = d.join("loam");
        if candidate.join("_index.md").is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(sub: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(sub)
    }

    #[test]
    fn resolves_bundle_from_nested_dir() {
        let start = fixture("sample_repo/src/deep");
        let got = resolve_bundle(&start).expect("should find the bundle root");
        assert_eq!(got, fixture("sample_repo/loam"));
    }

    #[test]
    fn resolves_when_starting_at_a_file_inside_the_tree() {
        let start = fixture("sample_repo/src/deep/marker.txt");
        let got = resolve_bundle(&start).expect("should find the bundle root from a file path");
        assert_eq!(got, fixture("sample_repo/loam"));
    }

    #[test]
    fn returns_none_outside_any_bundle() {
        // Root has no `loam/_index.md` ancestor on any sane system.
        assert!(resolve_bundle(Path::new("/")).is_none());
    }
}
