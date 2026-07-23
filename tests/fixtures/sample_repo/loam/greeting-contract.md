---
concept_id: greeting-contract
trust_tier: verified
claim_type: mechanical
sample_stream: census
sources:
  - path: src/greeting.rs
    content_hash: 5f0ac28c930cacb0ebb78747609658c08d7d283658856a6a91bdd18403646558
    span: [1, 3]
    quote: |
      pub fn greet(name: &str) -> String {
          format!("Hello, {name}!")
      }
provenance:
  extractor: fixture
  falsifiers: [anchor-model]
  resolution: cross_family
extracted_at: 2026-07-22T00:00:00Z
last_validated: 2026-07-22T00:00:00Z
---
# greeting-contract

`greet(name)` returns the string `Hello, <name>!`. See [index](_index.md).
