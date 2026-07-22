# Cross-family falsification is census-by-default; sampling is a window-bounded fallback

**Status:** accepted · 2026-07-22

Local GPU is a sunk fixed cost, so the T2 cross-family anchor model runs toward **census over all soft claims** rather than a statistical sample; metered cloud (T3) spend stays reserved for the small, genuinely-hard disagreement×criticality intersection. A deterministic **pre-flight feasibility estimate** — scannable-file count × per-file token estimate × measured GB10 throughput, compared against an overnight gardening window (provisionally ~8h) — decides *per corpus* whether full census fits. When it does not, the Supply §8 stratified/adaptive sampling apparatus engages as a **fallback** that spends the entire window's GPU-hour budget on falsification prioritized by **criticality × heat** — never a fixed 1% floor.

This is the deliberate rejection of the "verified backbone, soft claims advisory" alternative: on a sunk-cost box, "falsify more" is not a cost problem, it is the thesis.

## Considered options

- **A — Mechanical verified-backbone; soft claims advisory-by-default (`claimed`).** Rejected: it prices sunk-cost local compute as if it were metered API spend, and caps the product's ambition for no dollar saving.
- **B — Census-target soft-claim falsification, sampling as fallback (chosen).** Spends the fixed cost of the box to make "verified wiki" genuinely true for the reachable soft surface.

## Consequences

- **`claimed` is redefined** as a *transient queue state* ("not yet cross-family corroborated; queued"), not a permanent tier. The anchor drains it toward `corroborated` hot/critical-first across gardening passes. A cold, non-critical tail on a huge, actively-churning corpus may lag indefinitely *in practice*, but it is never *designed* as permanent. (Glossary updated.)
- **The Supply §8 "measurement, not protection" stance inverts** for the census regime: cross-family falsification *is* the protection for soft claims. Sampling is demoted from the default budget-allocator to (a) the huge-repo coverage-rationing fallback and (b) its still-valid measurement (extractor-precision estimation, kill criteria) and cluster-discovery (adapter curriculum) roles.
- **Required-but-deferred Supply PRD edits** (for the design/to-prd phase, not applied mid-grill): §6 gains the pre-flight sizing step; §8 is reframed (census default, sampling fallback, budget-filling not 1%-floor coverage); README "Architecture in One Paragraph" and Invariant 3 wording follow.
- **P0 impact:** *anchor claims-per-GPU-hour* is promoted from open-question §16.6 / §6-secondary-readout to a **P0 primary readout** — it sizes the gardening window and calibrates the pre-flight estimator, which is only sharp once P0 supplies real claim-density and tokens/file numbers.
- **Budget units (see ADR-0005):** the window budget is **overnight wall-clock / GPU-hours on sunk hardware, by intent.** The same pre-flight mechanism generalizes to a dollar budget only if a user deliberately runs a metered backend — an incidental, not the default.
- **Invariants preserved:** `corroborated` still requires cross-family agreement; same-base endorsement still cannot elevate above `claimed` (the T1.5 falsifier stays subtractive); dual sampling ledgers still never mix denominators; T3 cloud still fires only at disagreement×criticality.
