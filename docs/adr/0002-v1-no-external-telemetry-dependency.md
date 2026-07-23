# Loam v1 depends on no external telemetry consumer; feedback loops are P1+ amplifiers

**Status:** accepted · 2026-07-22

Loam v1's core value path — extract → verify → serve a verified wiki agents read instead of re-deriving — and the P0 token-savings proof (paired-run harness, Supply KC2 baseline arm) both require **no external telemetry consumer**. The five consumption feedback loops (heat→criticality, miss→demand, read-then-grep, outcome→live-economics, contradiction→demotion) are therefore reclassified from an assumed sensor layer to **P1+ amplifiers** that activate only when a downstream telemetry consumer drains the local spool. The spool schema is frozen in P0 so no signal is lost while the external Telemetry Event Contract remains pending.

ADR-0001 (census-by-default) is what makes this affordable: it removes *heat* from the critical path for small/moderate corpora, so the loops' absence costs v1 almost nothing.

## Considered options

- **Block v1 on the downstream consumer's intake contract.** Rejected — makes a separate product's roadmap a hard blocker on Loam shipping.
- **Build Loam-local fallbacks for each loop.** Rejected — duplicates a system that already exists externally for a v1 that doesn't need the loops closed.
- **Decouple: emit-only to the local spool, defer consumption (chosen).**

## Consequences — accepted v1 degradations (deferred, no local fallback)

- **Criticality is static-only** (S0 heuristics: import fan-in, churn, coverage, public-contract) without heat. In census regime heat is irrelevant; it only orders the sampling-regime queue on huge repos, where static criticality is a defensible proxy for exposure-if-wrong. Heat refines this in P1+.
- **Token economics is proven once** via the P0 paired-run benchmark (Supply KC2); continuous live-economics monitoring (Consumption KC3, outcome-joins) is P1+.
- **Staleness is content-hash-only** in v1 (fully local, no LLM, no external telemetry consumer); semantic-staleness detection via use-time demotion (Supply §14.5) is P1+.
- **Invariant preserved:** consuming agents still never write concepts; `loam observe` still writes to the inbox. That write boundary is independent of any external telemetry consumer.
