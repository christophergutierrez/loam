# The 60–80% rediscovery premise is a measured hypothesis, not an asserted fact

**Status:** accepted · 2026-07-22

The founding claim that agents spend 60–80% of task tokens on *rediscovery* is reframed from asserted fact (README, Supply §1) to a **hypothesis the P0 baseline arm measures**. The baseline arm's **discovery-vs-execution token split** is promoted to a P0 *primary* readout, because that split *is* the rediscovery share and therefore the **ceiling** on achievable savings. KC2's "≥30% reduction" target is a *distinct* quantity (achieved savings) and is sanity-checked against the measured ceiling **before** the treatment arm runs: a 30%-of-total target against, say, a 40% rediscovery ceiling implies capturing ~75% of all rediscovery, which is revisited rather than pursued.

Motivated by ADR-0002: with the paired-run benchmark now the sole proof of the token thesis, its premise must be measured, not assumed — and asserting an uncited number is precisely the "wrong fact trusted because it's stated confidently" failure the project exists to prevent.

## Considered options

- **Keep asserting 60–80% and target 30% blindly.** Rejected: risks a rigged-to-fail experiment (target may exceed ceiling) and contradicts the project's own anti-"trusted wrong fact" ethos.
- **Demote to a measured hypothesis; gate KC2 on the ceiling (chosen).**

## Consequences

- **Required-but-deferred edits:** README + Supply §1 reword 60–80% as a hypothesis-under-test; P0 §7 promotes the discovery/execution split from incidental ("where the harness can tell") to a primary readout; Supply §12 / Consumption §9 KC2 gains the ceiling sanity-check gate.
- **Discovery/execution split definition (resolved):** the split is measured at **tool-call-class** granularity — discovery = read-only exploration calls (grep/search/read/ls/cat) plus the reasoning turns ingesting their outputs; execution = edit/write/test-run/build calls plus surrounding reasoning; mixed reasoning turns are attributed both ways to yield a **conservative/liberal band**, so the rediscovery ceiling is reported as an *interval*, not a false point estimate (consistent with the project's measured-bounds-over-false-precision ethos). This is harness-agnostic (computable from any trace's tool-call records).
- **P0 prerequisite (cheap, must verify early):** the existing paired-run harness must emit **per-call token counts**, not only total-tokens-per-task, or the proxy is not computable. If it logs only totals, P0 adds that instrumentation before the baseline arm runs.
- Costs nothing extra: the baseline arm is already scoped in P0; this changes what it *reports*, not whether it runs.
