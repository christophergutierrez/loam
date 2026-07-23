# Anchor content-hash canonicalization is a cross-language seam contract

**Status:** accepted · 2026-07-22

The typed anchor's `content_hash` (Supply §7.1: path + content_hash + span + quote) is **written by the Python supply side and verified by the Rust demand side** (`loam-core`, ADR-0006). For a claim to stay `verified`/`corroborated` rather than flipping to STALE, both sides must compute *exactly the same bytes* for a given source span. That agreement is a contract, not an implementation detail, and it is easy to break silently — a CRLF file read with universal-newline translation on one side and raw bytes on the other yields different hashes, so every CRLF file would be permanently STALE. Recorded here so both implementations honor one definition.

## The canonical form

Given a source file and an inclusive 1-based line span `[start, end]`:

1. **Encoding:** read as UTF-8.
2. **Newline normalization:** replace `\r\n` and lone `\r` with `\n` **before** splitting. Line endings never affect the hash.
3. **Span extraction:** split the normalized text on `\n`; take lines `start..=end` (1-based, clamped to the available range); **join with a single `\n`** (no trailing newline).
4. **Hash:** `sha256` of the joined span's UTF-8 bytes, lowercase hex.

Notes: a trailing newline in the source produces a final empty element under `split('\n')`, which is excluded unless it falls inside the span; an out-of-range span hashes the empty string. The Rust implementation is `loam_core::concept::hash_span` (test: `hash_is_line_ending_invariant`).

## Consequences

- **The Python writer MUST implement the identical algorithm** (normalize newlines before hashing; join with `\n`; UTF-8; sha256 hex). This ADR is the spec it implements against; a cross-seam test should hash the same fixture from both sides and assert equality before Phase 1 produces a real bundle.
- Binary / non-UTF-8 source files are out of scope for anchoring in v1 (mechanical claims target text source).
- This mirrors the versioned-redaction/canonicalization discipline already used in TraceStore: state the canonical form once, test it from both sides.

## Considered alternatives

- **Hash raw bytes of the span** (no normalization). Rejected: makes the hash line-ending-dependent, so a benign CRLF↔LF conversion (git autocrlf, an editor) spuriously invalidates every anchor in the file.
- **Hash the stored `quote` instead of re-reading source.** Rejected: the quote is stored evidence and never changes, so it could never detect source drift — defeating the point of a content hash.
