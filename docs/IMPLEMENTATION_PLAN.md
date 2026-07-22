# Implementation Plan: Loam post-P0 build → v1 → roadmap (P0-remainder through P3)

> **Companion to `docs/P0_EXECUTION_PLAN.md`.** That plan reaches the P0 **gate decision**. This plan covers everything after it. **This entire plan is void unless P0 = PASS** (Supply KC1). Near phases (1–3, the buildable v1) are detailed; far phases (4–6 = P1–P3) are deliberately coarse and contingent — each is replanned via the PLAN loop once its predecessor lands. Detailing them now would be speculative.

## Planning Verdict
- **verdict:** READY_WITH_ASSUMPTIONS
- **task_tier:** full
- **tier_trigger:** persisted data (the OKF bundle is the product's canonical store), a public/consumed interface (the `loam` CLI + inbox schema), and a security surface (secret redaction) are all in scope → Blast-Radius (sec 16) forces full tier + human gates. No large-scale removal (greenfield), so migration/state-diagram machinery is minimal.
- **execution_policy:** cost_optimized
- **model_routing:** current-model-only
- **reason:** Consolidated multi-phase plan grounded in the reconciled PRDs + ADRs 0001–0006. READY_WITH_ASSUMPTIONS because it is (a) **gated on P0 = PASS**, and (b) **parameterized by P0 outputs that do not exist yet** — the winning T1 extractor + T2 anchor (E1), finalized per-stratum tolerances, criticality-band heuristics, and claim-density / anchor-claims-per-GPU-hour (c3). Those are listed assumptions; phases 1–3 are executable once P0 delivers them, phases 4–6 are coarse-by-design.

## Repository State (Staleness Contract)
- **HEAD:** `f652879` on branch `p0-planning` (the committed reconciled docs + ADRs + P0 plan). `main` is at `6311a1c`.
- **Discovered at:** 2026-07-22 (session).
- **Still greenfield:** no source code yet. Gate baselines are `unrun: greenfield` unless noted.
- **Executor pre-check:** verify P0 = PASS and load the P0 Results memo (E1) before starting; if any `docs/**`/`docs/adr/**` changed since `f652879`, re-validate cited facts.

## Preconditions (hard gates before Phase 1)
1. **P0 = PASS** — a local extractor cleared the ADR-0004 gate (soft strata + anchor/type ≤1%). If P0 killed/gated, this plan does not run.
2. **P0 outputs loaded** (from the E1 memo): winning T1 extractor + T2 anchor (with `models.lock`), finalized tolerances (`p0/tolerances.yaml`), criticality-band heuristics, claim-density + anchor-claims-per-GPU-hour, and the draft OKF schema shaken out during the P0 measurement harness.

## Requested Outcomes & Non-Goals
- **V1 (phases 1–3):** a real OKF bundle exists for the pilot repo (O-A); heterogeneous agents consume it through a Rust CLI with trust tiers + read-time staleness (O-B); the token thesis is proven on real work via the treatment arm (O-C).
- **Roadmap (phases 4–6):** the self-measuring gate (O-D, P1); the compounding loop — adapters, weakness registry (O-E, P2); advanced consumption — routing, promotion, distillation (O-F, P3).
- **Non-goals of *this* plan:** re-deriving anything P0 already decided; multi-user/team sync; real-time ingestion; cloud-hosted operation; anything a phase's predecessor hasn't unblocked.

## Facts, Assumptions, and Decisions
- **Stacks (ADR-0006):** Rust `loam-core`/CLI (demand); Python pipeline (supply); distinct across the artifact seam (bundle / SQLite index / inbox / spool) — no shared process/FFI.
- **Census-by-default (ADR-0001):** the supply pipeline falsifies soft claims toward census, pre-flight-sized; sampling is the huge-repo fallback.
- **No TraceStore in v1 (ADR-0002):** phases 1–3 emit to the local spool only; the feedback loops (heat/promotion/economics-live) are P1+ amplifiers that light up when TraceStore consumes the spool. The treatment-arm economics (Phase 3) use the local spool + paired-run harness, TraceStore-independent.
- **Assumption:** P0's winning extractor is stock gen-0; adapters (S2 T1.5 cheap-falsifier, triage model) are P2 — so the Phase-1 pipeline runs S0 (deterministic triage) → S1 (stock extraction) → S3 (anchor falsification) → S4 (write); **S2 is skipped until P2**.

## Outcome Traceability (phase-level; milestone-level detail in phases 1–3)
| outcome | phase | gated by | notes |
| --- | --- | --- | --- |
| O-A real OKF bundle | 1 | P0 PASS + outputs | bundle schema is a blast-radius human gate |
| O-B CLI consumption | 2 | Phase 1 (a bundle to read) | CLI contract is a blast-radius human gate |
| O-C token thesis on real work | 3 | Phase 2 (live CLI) + P0 baseline arm | closes Supply KC2 |
| O-D self-measuring gate (P1) | 4 | v1 (phases 1–3) + P0 tolerances | coarse |
| O-E compounding/adapters (P2) | 5 | P1 | coarse; PEFT/Qwen3.6 risk (Supply §16.1) |
| O-F advanced consumption (P3) | 6 | P2 + TraceStore intake | coarse |

**Consumption P-track mapping (two P-numberings exist).** Supply's P0–P3 drive the plan phases above; the Consumption PRD §10 has its *own* P-track. Reconciliation: Consumption-P0 (files+CLI core) = **Phase 2**; Consumption-P1 (loop consumers, consultation dashboards, remaining stanzas) = **Phase 4**; Consumption-P2 (lint mechanical+semantic) = **Phase 4 mechanical / Phase 5 semantic**; Consumption-P3 (MCP wrapper, `bundle` ranking) = **Phase 6**. This mapping exists so no Consumption deliverable falls through the cracks.

## Final-State Invariants (carried from the non-negotiables; re-checked at each phase end)
```yaml
- id: INV-evidence-contract   # every admitted claim has a typed anchor (path+hash+span+quote); no anchor, no admission
  category: presence
  scope: phase-end
  check: writer rejects a claim with no resolving anchor; bundle scan finds zero anchorless concepts
- id: INV-anchor-independence  # corroborated requires cross-family; same-base never elevates above claimed
  category: absence
  scope: phase-end
  check: no concept reaches corroborated without a cross-family endorsement in provenance.falsifiers
- id: INV-exact-binomial       # Wald/normal banned wherever intervals appear (esp. Phase 4)
  category: absence
  scope: final
  check: k=0 → upper ≈ 3/n; reference CP points; no normal-approx in code
- id: INV-dual-ledgers         # random and directed sample_stream denominators never mix (Phase 4+)
  category: absence
  scope: phase-end
  check: rate estimates draw only sample_stream==random rows
- id: INV-no-secrets           # secrets never enter any surface incl. prompt logs; redact source before prompt
  category: absence
  scope: every-pass
  check: gitleaks-class scan of bundle + index + spool + serving logs clean; seeded fixture secret caught pre-redaction
- id: INV-derived-rebuildable  # anchor index, spool, caches are rebuildable and never committed; markdown bundle canonical
  category: absence
  scope: phase-end
  check: .gitignore covers *.sqlite/spool/caches; a fresh index rebuild from the bundle reproduces get/search results
- id: INV-agents-never-write   # consuming agents write only to the inbox, never concepts
  category: absence
  scope: every-pass
  check: loam-core has no concept-write path; observe writes only under the inbox dir
```

---

## Phase 1 — Supply build: OKF bundle + stock pipeline → a real bundle  *(FULL detail; P0 §2 step 4)*
- **objective:** produce a real, trust-tiered OKF bundle for the pilot repo from the stock pipeline. **prerequisites:** P0 PASS + outputs. **blast_radius:** the OKF concept schema is *persisted data* → **human gate on the schema freeze (M1.1)**. **rollback_boundary:** delete the generated bundle + index (both rebuildable). **exit_gate:** M1.1–M1.5.

- **M1.1 — OKF concept schema + writer + validator** *(human-gated: schema freeze)*. Frontmatter per Supply §4 (`concept_id`, `sources[path/content_hash/span]`, `trust_tier`, `claim_type`, `provenance{extractor,falsifiers,resolution}`, `sample_stream`, timestamps). **gate:** a concept round-trips write→parse→validate; **a claim with no resolving anchor is rejected** (INV-evidence-contract); **the frozen schema is human-ratified before exit** (blast-radius: persisted data). baseline: no writer → fails.
- **M1.2 — S0 deterministic triage** (reuse P0 criticality-band heuristics; **no triage model — that's P2**). **gate:** emits routing metadata (class/criticality/route) per file and **emits zero claims**; baseline: no triage → fails.
- **M1.3 — S1 extraction (stock winning extractor) + redaction-before-prompt.** Reuse **P0 a4 (frozen prompt + decode config) + c1 (extraction runner) + the winning extractor in `models.lock`** for extraction; reuse the **a2 harness only for the redaction-before-prompt detector** (a2 is the mechanical/redaction harness, not the extractor). **gate:** claims cached with typed anchors + self-signals for the target set; redaction scan of every write surface clean (INV-no-secrets); baseline: no claims → fails.
- **M1.4 — S3 anchor falsification + mechanical verification + resolution.** On the (small) pilot repo Phase 1 runs **census unconditionally** (ADR-0001); **the pre-flight feasibility estimator and the sampling-regime fallback are Phase 4 (P1)** — M1.4 must not depend on them. **S2 (T1.5 cheap falsifier) is skipped until P2.** **gate:** soft claims cross-family-falsified (census), mechanical claims verified deterministically; disagreements route to resolution (mechanical / cloud / human); baseline: nothing falsified → fails.
- **M1.5 — S4 write + compound + index** *(TRACER BULLET is a thin M1.2→M1.5 slice on ONE file done first)*. Write concepts with trust tier + full provenance; **census concepts carry `sample_stream: census`, mechanical-only concepts `unsampled` (never absent — mirrors P0 INV-census-only, so P1 dual-ledger tooling can't misread them)**; the **compound step generates inter-concept cross-links and a bundle-root index concept** (Consumption §3 Tier A entry point + the link graph `loam bundle` traverses); build the derived SQLite anchor index; **conflict objects are first-class**. **gate:** a real bundle exists in git; index rebuilds from it (INV-derived-rebuildable); tiers assigned; the bundle-root index concept + inter-concept links exist; **a doc-vs-code disagreement — naturally occurring or a seeded synthetic fixture (as P0 seeds the tripwire) — is stored as a conflict object, not dropped**; baseline: no bundle → fails.

**Tracer bullet (do first):** triage 1 file → extract → verify+falsify → write 1 concept → build index → read it back through the index. Proves S0→S4 end-to-end before fleshing any stage.

## Phase 2 — Consumption build: `loam-core` (Rust) + CLI  *(FULL detail; ADR-0006)*
- **objective:** the demand surface over the Phase-1 bundle. **prerequisites:** Phase 1 (a bundle to read). **blast_radius:** the CLI + inbox schema are *public/consumed interfaces* → **human gate on the CLI contract + inbox schema**. **rollback_boundary:** the CLI is additive; uninstall the binary. **exit_gate:** M2.1–M2.7.
- **Command scope:** this phase builds four of the five commands — `get`, `search`, `bundle`, `observe`. **`loam lint` is deferred** (Consumption §10 places lint at its own P2): mechanical-tier lint lands in Phase 4 (it needs only the Phase-1 anchor index), semantic-tier lint in Phase 5 (it *is* the T1.5 falsifier adapter run in reverse, Consumption §8.2). The v1 stanzas (M2.7) therefore ship **only the instructions for commands that exist**.

- **M2.1 — `loam-core` skeleton + corpus resolution** (nearest-ancestor bundle discovery, workspace-config override). **gate:** resolves the bundle root deterministically from a nested cwd; baseline: no resolver → fails.
- **M2.2 — telemetry spool** (append-only SQLite, non-blocking, never-drop; spool-only degraded mode per ADR-0002) + `--json` scaffolding. **Built before the commands that emit to it** (fixes the ordering: every command's gate below asserts a spool emission). **gate:** an event appends and survives; **a write never blocks/fails when TraceStore is absent** (kill TraceStore, emit, must succeed); baseline: no spool → fails.
- **M2.3 — `loam get`** *(TRACER BULLET: read → hash-verify → **emit to spool** end-to-end via the CLI — crosses the bundle+index AND spool seams)*. Read-time content-hash verification, inline **STALE** marker + changed-anchor list, trust tier always shown, `concept_read` event emitted. **gate:** returns a concept; a dirtied anchor yields STALE inline; hash verification is sub-perceptible; the read emits a `concept_read` event to the spool; baseline: no get → fails.
- **M2.4 — `loam search`** (text/frontmatter over the bundle). **gate:** returns matches; a **zero-result search emits a `search_miss` event** to the spool; baseline: no search → fails.
- **M2.5 — `loam bundle <task>`** dumb-first (index + explicit link traversal from the M1.5 link graph, size-capped, tiers surfaced; Consumption §5.3). **gate:** assembles a linked-concept set within the cap **by traversing the M1.5 cross-links from the bundle-root index concept**; emits `bundle_assembled` composition; baseline: no bundle cmd → fails.
- **M2.6 — `loam observe` → inbox** (typed: claim/contradiction/concept-wrong/concept-missing/procedural). **gate:** writes a typed inbox entry with harness/task/evidence provenance and **never writes a concept** (INV-agents-never-write); emits `observation_filed`; baseline: no observe → fails.
- **M2.7 — instruction stanzas** (CLAUDE.md/AGENTS.md block + Claude Code Skill + Hermes) generated via `loam init --refresh`. **gate:** stanzas ship only the instructions for **built** commands (bundle-before-explore, tier semantics, observe findings, STALE=check-source) and **do NOT instruct `loam lint` until lint exists** (Phase 4+); baseline: no stanzas → fails.

## Phase 3 — Close KC2 (treatment arm)  *(detailed; Supply KC2 / Consumption KC3 one-time proof)*
- **objective:** prove the token thesis on real work. **prerequisites:** Phase 2 (live CLI) + the P0 baseline arm (frozen d1 task set + baseline token table). **rollback_boundary:** discard treatment outputs. **exit_gate:** M3.1–M3.2.
- **M3.1 — treatment paired runs** (same task classes as P0 d1, live bundle, real CLI). **gate:** treatment token table produced, matched by task class to the baseline; baseline: no runs → fails.
- **M3.2 — baseline-vs-treatment economics** (TraceStore-independent: local spool + paired-run harness). **gate:** token delta computed vs the frozen P0 baseline ceiling; check ≥30% (or the d3-revised target); consultation-before-exploration rate measured from the spool. **baseline_polarity:** target inherits the d3 ceiling check (ADR-0003) — a delta below the target is an honest fail, surfaced, not massaged.

---

## Phase 4 — P1: the self-measuring gate  *(COARSE; contingent on v1 + P0 tolerances)*
Sampling engine (§8: strata, tolerance-derived sizes, Clopper-Pearson + Beta-binomial cross-batch, **dual ledgers**, directed pursuit + SPRT, adaptive escalation, and the **three-state broken-stratum tripwire** from ADR-0004); **the pre-flight feasibility estimator + sampling-regime fallback (ADR-0001)** — deferred here from Phase 1, which ran census-only on the small pilot; seeded-mutation audit (§11) with gate-recall measurement; fuller conflict objects + human queue; trust-tier promotion/demotion scaffolding (the mechanism, not yet the TraceStore-driven signal); dirty-set incremental gardening; **`loam lint` mechanical tier** (Consumption §8.1 — needs only the Phase-1 anchor index; add the lint-before-done stanza instruction once it exists); **Consumption-P1 items** — miss/heat/outcome loop consumers, consultation-rate dashboards, remaining harness stanzas. **Invariants foregrounded:** INV-exact-binomial, INV-dual-ledgers. Replan via PLAN once Phase 3 lands and P0 tolerances are finalized.

## Phase 5 — P2: compounding  *(COARSE; contingent on P1; research-heavy)*
Extraction / triage / T1.5-falsifier **QLoRA adapters** + the §10 batch training loop with the audit-gate (seeded-mutation + stock-vs-tuned regression); weakness registry consulted by S0; unexplained bucket + abductive coordinate proposal; T1.5 multi-LoRA serving (enables S2 in the pipeline); **`loam lint` semantic tier** (Consumption §8.2 — the T1.5 falsifier adapter run in reverse; gated on the T1.5 adapter existing); TraceStore trace ingestion into the corpus. **Open risk (Supply §16.1):** PEFT/QLoRA target-module support for Qwen3.6 Gated-DeltaNet — a spike before committing the base. Coarse by design; replan after P1.

## Phase 6 — P3: advanced consumption  *(COARSE; contingent on P2 + TraceStore intake)*
Bundle router / ranking layer (earned by miss telemetry, Consumption §5.3); use-time promotion/demotion **wiring** (needs TraceStore consuming the spool — the ADR-0002 amplifier); skill-graduation pipeline (procedural → skills via the seven-metric eval); **MCP server wrapper (Consumption Tier C, §3 — data-gated: built per-harness only if consultation stays low after instruction iteration)**; CAG hot-core prefix cache; LoRA weight-baking of the verified core (Amesh). Coarse; each item is independently gated and replanned when its inputs exist.

---

## Consolidated Verification (v1 = phases 1–3)
Loam v1 is done when: a real trust-tiered OKF bundle exists (INV-evidence-contract, INV-anchor-independence); the Rust CLI serves `get`/`search`/`bundle`/`observe` with read-time staleness, tiers, and spool telemetry that never blocks (INV-agents-never-write, INV-derived-rebuildable); no secret entered any surface (INV-no-secrets); and the treatment arm shows the KC2 token delta on real work against the frozen P0 baseline.

## Blast-Radius Decisions (human gates)
- **P0 = PASS** — the precondition for the entire plan.
- **OKF concept schema freeze (M1.1)** — persisted data / canonical product surface.
- **CLI + inbox schema (Phase 2)** — public/consumed interface across the artifact seam (ADR-0006).
- **Redaction on the real bundle (INV-no-secrets)** — security; redact-before-prompt, serving logs disabled.

## Replan Triggers
- P0 killed/gated → this plan is void; act on the P0 memo's revised thesis instead.
- P0 outputs differ materially from assumptions (e.g. tolerances far tighter, claim density far lower) → re-validate phase 1/4 sizing.
- PEFT/Qwen3.6 spike (Phase 5) fails → fall back to the §16.1 base (Qwen3-30B-A3B-Instruct-2507) and replan P2.
- TraceStore intake contract still absent when Phase 6 starts → Phase 6 promotion/economics-live items stay parked (spool accumulates); the rest of P3 proceeds.
- Any spec file changed since `f652879` → re-validate.

## Review Record
- **Pass 1 — draft:** Lead Planner. Full tier; near phases (1–3) milestone-detailed with falsifiable gates, far phases (4–6) coarse-and-contingent by design.
- **Pass 2 — independent review:** two reviewers spawned. The completeness/sequencing/alignment reviewer completed (independent); the gate-quality/blast-radius reviewer **died on an API error mid-run**, so its lens was completed **inline** (PLAN sec-13 degradation) → `gate_audit: mixed (independent completeness + inline gate)`. **0 Blocking, 5 Material, 5 Minor — all accepted and applied:**
  - **[Material] `loam lint` dropped from the whole roadmap** — the 5th CLI command + Consumption KC4, and M2.7 shipped a "lint-before-done" stanza for a nonexistent command → **placed lint: mechanical tier in Phase 4, semantic tier (T1.5-in-reverse) in Phase 5; v1 stanzas no longer instruct lint until it exists.**
  - **[Material] spool sequenced after its consumers** — M2.2–M2.4 gates emit to a spool built at M2.6 → **spool moved to M2.2 (before the commands); the `get` tracer now crosses the spool seam.**
  - **[Material] `bundle` traversal needs links Phase 1 didn't create** → **M1.5 compound step now generates inter-concept cross-links + a bundle-root index concept.**
  - **[Material] pre-flight estimator forward-dependency** — M1.4 referenced "pre-flight-sized" census/sampling but the estimator + sampling fallback are Phase 4 → **Phase 1 clarified as census-only on the small pilot; estimator + fallback deferred to Phase 4.**
  - **[Material] Consumption P-track unplaced** (MCP, dashboards, remaining stanzas) → **added the Consumption-P-track mapping note; folded MCP → Phase 6, loops/dashboards → Phase 4.**
  - **[Minor] M1.3 cited the wrong P0 harness** (a2 is mechanical/redaction, not the extractor) → **cite a4 + c1 + models.lock for extraction; a2 for redaction only.**
  - **[Minor] Phase-1 `sample_stream` value unstated** → **census concepts write `sample_stream: census`, mechanical `unsampled` (never absent).**
  - **[Minor] conflict-object gate assumed a natural conflict** → **a seeded synthetic conflict fixture satisfies the gate (as P0 seeds the tripwire).**
  - **[Minor] M1.1 schema-freeze gate not explicit** → **human ratification of the frozen schema is now an M1.1 exit condition.**
  - **[Minor] `--json`/spool wiring** folded into the M2.2 spool foundation.
- **Reviewer positives recorded:** the phase DAG (1→2→3, 4–6 gated on predecessor) is acyclic/reachable; P0 outputs are correctly consumed as preconditions; S2/T1.5 correctly deferred to Phase 5; the M1.5 and M2.3 tracers are genuinely vertical; no ADR or non-negotiable contradicted; the coarse treatment of 4–6 is appropriate.
- **Convergence:** pass 2 resolved 10, introduced 0, regressed 0. No Blocking/Material open. Verdict holds: READY_WITH_ASSUMPTIONS (gated on P0 = PASS + P0 outputs; far phases coarse-by-design).

```json
{
  "verdict": "READY_WITH_ASSUMPTIONS",
  "task_tier": "full",
  "execution_policy": "cost_optimized",
  "model_routing": "current-model-only",
  "passes": 2,
  "open_blocking_findings": 0,
  "open_material_findings": 0,
  "gate_audit": "mixed (independent completeness reviewer + inline gate-quality/blast-radius audit; the gate reviewer subagent died on an API error)",
  "staleness": { "head": "f652879", "branch": "p0-planning", "discovered_at": "2026-07-22" },
  "preconditions": ["P0 = PASS", "P0 outputs loaded (winning models, tolerances, bands, claim-density)"],
  "blast_radius_decisions": ["P0 PASS precondition","OKF schema freeze (M1.1)","CLI + inbox schema (Phase 2)","redaction on real bundle (INV-no-secrets)"],
  "human_decisions_required": ["P0 gate outcome","OKF schema freeze","CLI/inbox contract"],
  "coarse_phases": ["4 (P1)","5 (P2)","6 (P3)"],
  "plan_location": "docs/IMPLEMENTATION_PLAN.md",
  "summary": "Consolidated post-P0 plan: phases 1-3 (supply build -> Rust CLI consumption incl. spool-first ordering -> KC2 treatment arm) detailed and executable once P0 passes; phases 4-6 (P1 gate + pre-flight estimator + mechanical lint, P2 compounding + semantic lint, P3 advanced consumption + MCP) coarse and contingent. Pass-2 review fixed 5 Material + 5 Minor incl. the dropped loam lint command; none open."
}
```
