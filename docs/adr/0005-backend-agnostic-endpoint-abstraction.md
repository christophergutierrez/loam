# Backend-agnostic inference via an endpoint abstraction; sunk-cost local remains the intent

**Status:** accepted · 2026-07-22

Loam's motivating and intended deployment is unchanged: **sunk-cost local inference on owned hardware** — the origin of the "pay one slow pass, then coast" thesis and what makes census-by-default (ADR-0001) economically free. The architecture imposes **no hard hardware or model-checkpoint requirement**. The pipeline talks to inference **endpoints through a uniform (OpenAI-compatible-style) abstraction**, motivated by the common real-world case that a **proxy fronts the owned hardware**. Tiers are defined by role + required properties — T0 small dense; T1 sparse-MoE, adapter-carrying, fits the box's KV headroom; T2 *different base family from T1*, stock, strong entailment — not by named checkpoints. P0's model-floor experiment is the selection mechanism.

Hardware and model names in the docs (DGX Spark/GB10, sm_121, NVFP4/MXFP4, Qwen3.6-35B-A3B, GPT-OSS-120B, Nemotron 3 Nano) are **"e.g." reference config, never dictated** — owned hardware may equally be H200s, a large Mac, etc. Quantization flags, KV sizing, and vLLM image digests are per-backend deployment-recipe details, not core design.

Because every tier sits behind the endpoint abstraction, a user *could* point one at a metered cloud API (e.g., Haiku). This is a **trivial incidental, explicitly not the intent**: census-by-default assumes sunk cost, so a metered backend is the operator's own coverage-vs-dollars tradeoff (they would configure the sampling regime), outside the designed path.

## Considered options

- **Hard-pin GB10 + the model slate.** Rejected — needless coupling; blocks proxy-fronted and heterogeneous owned hardware.
- **Fully deployment-neutral, metered as a first-class target.** Rejected — throws away the sunk-cost differentiator and the census economics; metered is incidental, not a goal.
- **Endpoint abstraction; sunk-cost-and-proxy as the designed path; metered incidental (chosen).**

## Consequences

- **Cross-family invariant holds regardless of backend:** T1 and T2 must be different base families (anchor independence) no matter what fronts them. Whether a cross-family pair *stands up at all on the chosen backend* is an explicit **P0 shakedown gate** (promoted from the §6 side-benefit).
- **Census tilts the T2 anchor toward throughput** (ADR-0001 makes it falsify *every* soft claim): favors a lighter/faster cross-family model resident (e.g., Nemotron-class) with an optional heavier nightly pass (e.g., 120B-class) — a P0 selection criterion, not a fixed choice.
- **Required-but-deferred edits:** README + CLAUDE.md (GB10 → reference config while keeping sunk-cost intent); Supply frontmatter "Target platform", §1.1 "why now", §5 model table + GB10 sizing, §5.5, §6 "favorable regime", §16 open questions; Glossary GB10/Workhorse/Anchor entries updated to reference-config (done).
- **Invariants preserved:** T2 anchor stays stock; anchor independence; exact-binomial; dual ledgers.
