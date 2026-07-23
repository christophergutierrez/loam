---
concept_id: stale-example
trust_tier: claimed
claim_type: soft
sample_stream: census
sources:
  - path: src/greeting.rs
    content_hash: 0000000000000000000000000000000000000000000000000000000000000000
    span: [1, 3]
    quote: |
      (intentionally wrong hash — this concept is STALE by construction, to
      exercise read-time hash verification without mutating a committed source)
extracted_at: 2026-07-22T00:00:00Z
last_validated: 2026-07-22T00:00:00Z
---
# stale-example

A concept whose anchor hash no longer matches source — always STALE.
