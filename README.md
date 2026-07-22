# Loam

**Local extraction, verified knowledge, compounding agents.**

Loam is a knowledge system for AI agents built on one economic observation and one design commitment. The observation: agents spend 60–80% of their tokens *rediscovering* corpora that previous sessions fully understood and then forgot, and local inference on owned hardware (DGX Spark / GB10) makes exhaustive background extraction a fixed cost instead of a metered one. The commitment: **the write path is the product** — knowledge is admitted only through a verification gate whose error rate is itself continuously measured, because a wrong fact an agent trusts is worse than no fact at all.

The steady state: pay one slow local pass per corpus, then coast — agents read a verified wiki at near-zero marginal cost instead of re-deriving the world every session, and every task they run feeds trust signals back into the store.

## Documents

| Document | Scope |
|---|---|
| [Supply PRD](./docs/SUPPLY_PRD.md) | **Supply side.** Extraction pipeline, verification tiers, storage (OKF bundle), model/adapter topology on GB10, sampling design, adapter training loop, gate audits, kill criteria. |
| [Consumption PRD](./docs/CONSUMPTION_PRD.md) | **Demand side.** How heterogeneous agents (Claude Code, Codex, Grok Build, Antigravity, Hermes, …) find, trust, use, and improve the store: integration tiers, the `loam` CLI, telemetry feedback loops, the write boundary, and the contradiction linter. |
| [Glossary](./docs/GLOSSARY.md) | Canonical definitions of every coined term, including the deliberately disambiguated overloads (tier vs. stage, evidence anchor vs. anchor model, OKF bundle vs. task bundle). |
| [P0 Experiment Protocol](./docs/P0_EXPERIMENT_PROTOCOL.md) | The gate before all building: labeled-sample design, labeling rubric, per-stratum tolerances and derived sample sizes, the model-floor experiment, and the cross-PRD P0 dependency ordering. |

The two documents form a closed system: every mechanism in one has its sensor or consumer in the other (mapped in Consumption §11).

## Architecture in One Paragraph

A phased batch pipeline on a single GB10 sweeps a corpus: deterministic heuristics plus a tiny model triage every file (routing metadata only, no claims); a sparse-MoE workhorse (Qwen3.6-35B-A3B + QLoRA adapters) extracts claims with typed evidence anchors; a same-base falsifier adapter cheaply kills sloppy errors; a stock cross-family anchor model (GPT-OSS / Nemotron) falsifies a stratified, adaptively-escalating sample (exact-binomial math, 100% of critical files); disagreements resolve mechanically where possible, by cloud model only at the small intersection of disagreement × criticality. Surviving claims write to an OKF-shaped markdown wiki in git, with content-hash provenance, trust tiers, and declarative secret redaction. Agents consume it through a five-command CLI (`get`, `search`, `bundle`, `observe`, `lint`) that emits telemetry to TraceStore; heat, misses, contradictions, and task outcomes flow back to drive re-extraction, criticality promotion, trust demotion, and the next adapter generation's training curriculum.

## Invariants (the arguments already settled)

1. **Write path is the product.** No claim enters without a typed evidence anchor; mechanical verification precedes LLM verification; trust tiers (`verified` > `corroborated` > `claimed`) are always surfaced to consumers.
2. **Anchor independence is sacred.** The cross-family falsifier stays stock. Same-base endorsement (any Qwen judging any Qwen) can never raise a claim above `claimed` — the same-base falsifier is purely subtractive (it kills claims, it elevates nothing); shared weights share blind spots, and LoRA cannot drift far enough to escape them. `corroborated` requires cross-family agreement by definition.
3. **Dual sampling ledgers.** Random stream measures; directed stream hunts. Denominators never mix, or the health metrics silently corrupt.
4. **Exact binomial, never normal approximation.** Zero-failure bounds (Rule of Three), Clopper-Pearson intervals, Beta-binomial posteriors across batches. Wald intervals are banned in the rare-error regime.
5. **Agents propose; the pipeline disposes.** No consuming agent writes concepts. Observations are untrusted hints entering an inbox; evidence is always re-derived from source.
6. **Advisory lint, self-protecting store.** The linter offers resolution at the moment of maximum context; ignored warnings still demote and queue the affected concepts. (CI/CD enforcement: explicitly future scope.)
7. **Secrets never enter any surface.** Deterministic detection at T1; declarative replacement tokens (never masks, which re-trigger scanners); registry audit; redaction upstream of caches, telemetry, and training pools.
8. **Kill criteria are armed.** Both PRDs carry falsifiable thresholds (extraction precision, token delta, staleness bound, gate recall; consultation, bundle fallback rate, live economics, lint precision). Failing them kills or gates the project — that is what they are for.

## Status and Next Steps

Both PRDs are at v0.1 (draft), the glossary and the **P0 Experiment Protocol** are drafted, and the immediate next step is executing P0: label the ~100-file sample and run the model-floor experiment that tests kill criterion 1 before anything else is built. Component TRDs follow P0 results, not precede them.

## Related In-House Systems

- **TraceStore** — separate product; system of record for agent traces and Loam telemetry events. Loam emits; TraceStore records; the pipeline queries.
- **Killhouse** — origin of the falsifiable-gate and seeded-mutation audit patterns used in the Adversarial Gate Audit.
- **Skill evaluation framework** — decides when recurring procedural knowledge graduates from wiki concepts to skills (paired-run token benchmarking).
- **Amesh LoRA pipeline** — eventual weight-baking target for the most stable verified core.
