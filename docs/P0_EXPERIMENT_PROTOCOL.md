# Loam P0 Experiment Protocol — v0.1

**The gate before all building: does any local extractor clear the tolerances?**

| | |
|---|---|
| Status | Draft v0.1 |
| Owner | Chris Gutierrez |
| Tests | Supply kill criterion 1 (extraction precision); establishes the kill criterion 2 baseline arm |
| Companion to | Supply PRD §5.5, §8, §12, §15; Consumption PRD §10 |

---

## 1. Purpose and decision

One question, answered before component TRDs exist: **on a hand-labeled sample, does the best local extractor's per-stratum precision clear the per-stratum tolerances of §5 below?**

- **Pass** → the sunk-cost thesis holds; the winning model claims the T1 tier; P0 builds proceed.
- **Fail (no local model clears)** → the project is killed or gated on a revised thesis (bigger anchor share, cloud-heavier resolution, or narrowed claim ontology). That decision is made explicitly, not by drift.

Everything here is sized so the experiment is one to two weeks of part-time work plus overnight GPU batches — cheap enough that nobody is tempted to skip it, real enough that its numbers are binding.

## 2. Cross-PRD P0 dependency ordering

The two PRDs' P0 phases interleave. Binding order:

1. **This protocol executes first.** Needs: the labeled sample (§4), the mechanical verification harness (Supply P0), stock T1/T2 models serving on the GB10. Needs **no** consumption layer, **no** TraceStore intake.
2. **Kill criterion 2, baseline arm** runs in parallel with step 1: pre-Loam token counts on the matched task set (§7). Requires only the existing paired-run harness — no wiki exists yet, which is the point.
3. **Gate decision** on kill criterion 1. Nothing below this line starts until it passes.
4. **Supply P0 remainder**: OKF bundle schema finalized, stock-pipeline run over the pilot repo → a real bundle exists.
5. **Consumption P0**: `loam-core`, `get`/`search`/`bundle`/`observe`, corpus resolution, stanzas. Telemetry runs **spool-only** (Consumption §7.1 durability) — the TraceStore intake surface (Supply §16.4) is *not* a P0 blocker; flush lands when the intake contract does.
6. **Kill criterion 2, treatment arm**: paired runs against the live bundle via the real CLI. Baseline (step 2) vs. treatment (this step) closes the token-economics measurement.

The one genuinely external dependency is the **Telemetry Event Contract** (Loam↔TraceStore boundary). It blocks *flushing*, not P0 progress; drafting it belongs to whoever touches TraceStore next.

## 3. Pilot corpus and sample frame

- **Pilot corpus**: one in-house repo of moderate size (5–50K files is fine; the sample is what's bounded, not the repo). Must contain real code, real configs, and at least some authoritative internal docs — all three claim surfaces.
- **Sample frame**: all files after S0-style deterministic triage assigns criticality bands (import fan-in, churn, coverage, size, public-contract detection — heuristics only for P0; no triage model exists yet and none is needed).
- **Sample**: ~100 files, stratified per §5, drawn randomly *within* strata. File list is frozen and committed before any model runs (`p0/sample_manifest.json`: paths + content hashes). Post-freeze edits to sampled files drop the file (recorded, not swapped silently).

## 4. Labeling rubric

**Unit of scoring is the claim, not the file.** A ~100-file sample at realistic claim density (5–15 claims/file) yields roughly 800–1,200 scored claims — that budget is what makes the §5 sample sizes achievable, and files were chosen as the sampling unit only because extraction is file-scoped.

**Procedure per extracted claim:**

1. **Anchor check (mechanical, first).** Does the anchor resolve — path exists, hash matches the frozen manifest, span contains the quoted evidence? Anchor failure = claim scored **wrong**, subtype `bad_anchor`, regardless of semantic content.
2. **Type check.** Is the claim correctly typed `mechanical` vs. `soft` per the glossary definitions? Mistyped = **wrong**, subtype `bad_type` (a mistyped soft claim would dodge the falsification budget forever — this is not pedantry).
3. **Truth check.**
   - `mechanical` claims: verified by the harness (tree-sitter / import analysis / schema introspection). Harness verdict is final.
   - `soft` claims: human judgment against the anchored span *plus* whatever context the labeler needs. Scale: **correct / wrong / unsupported** — `unsupported` means plausibly true but the anchored evidence doesn't establish it; it scores as **wrong** (the evidence contract is the product).
4. **Completeness is not scored.** P0 measures precision only. Recall (did the extractor miss claims a labeler would want?) is recorded as free-text notes where obvious, feeding the ontology (Supply §16.3), but never enters pass/fail math.

**Labeler discipline:** every claim labeled by one primary labeler; a random 15% (plus every claim the primary marked borderline) labeled independently by a second. Disagreement → adjudication note; if adjudicated disagreement rate exceeds 5%, the rubric is ambiguous — fix the rubric and relabel the affected subtype before scoring any model. Labels are committed (`p0/labels/`) before per-model results are compared, and never edited afterward.

## 5. Strata, tolerances, and the sample-size math

P0 strata are deliberately coarser than production strata (Supply §8.1) — criticality band × claim type — because ~1,000 claims can't power a file-type × route matrix, and the decision doesn't need one.

| Stratum | Tolerance (provisional) | Clean claims needed (Rule of Three, 95%) | Expected supply from ~100 files |
|---|---|---|---|
| critical × mechanical | 1% | 300 | ~250–400 (critical files are claim-dense; harness-scored, so cheap to label) |
| critical × soft | 3% | 100 | ~100–200 |
| standard × mechanical | 3% | 100 | ~200–300 |
| standard × soft | 5% | 60 | ~150–250 |

Sampling weights: ~60 of the ~100 files drawn from the critical band (oversampled relative to corpus share, since its tolerances are tightest), ~40 from standard. The `low` band is excluded from P0 entirely — its tolerance would be loose enough that it cannot change the gate decision.

**Rules that make the numbers real rather than decorative:**

1. **Tolerances are the primitive.** They are provisional here and finalized (or revised with a written rationale) as the first section of any v0.2 of this protocol. Sample sizes are always *derived* — nobody negotiates n directly.
2. **Pass per stratum** = the observed Clopper-Pearson 95% upper bound on error rate ≤ tolerance. With zero observed errors this reduces to the Rule of Three column; with errors observed, the exact interval decides — no special-casing.
3. **Undersupplied stratum** (sample yields fewer claims than the required n): extend the sample with additional files from that stratum *drawn by the same frozen-manifest procedure* — never by relaxing the tolerance mid-experiment. If extension is impractical, the stratum reports its achievable bound and the gate decision must call this out explicitly.
4. **All P0 labels are `census` stream.** Nothing here enters the production random-stream ledgers; the dual-ledger rule (Supply §8.5) applies from P1 onward.

## 6. Model-floor experiment

**Candidates** (exact HF checkpoint IDs and vLLM image digests recorded in `p0/models.lock` before the first run — family names are not deployable identifiers):

- Qwen3.5-4B dense (the "is small enough?" floor probe)
- Qwen3.6-35B-A3B (the presumptive T1 workhorse)
- One large reference: GPT-OSS-120B (upper bound on what local can do; doubles as anchor-model shakedown)

**Conditions, identical across candidates:** stock weights (generation 0 — no adapters exist), the same full extraction prompt with ontology v0 and few-shot examples, the same evidence-context recipe per file (file-alone for P0; context recipes are a P2 variable), temperature 0 or fixed low, one run per candidate (self-consistency voting is an adapter-era luxury).

**Scoring:** every candidate's claims over the same frozen sample, labeled per §4. Mechanical claims are auto-scored by the harness; human labeling effort concentrates on soft claims. Per-candidate, per-stratum precision with exact intervals.

**Decision rule (Supply §5.5, made concrete):** the smallest candidate that passes *all four strata* wins the T1 tier. If only the 120B-class passes, T1 economics need rework before proceeding (that is a *gate*, not a kill). If no local candidate passes any configuration, kill criterion 1 has fired.

**Secondary readouts (recorded, not gating):** schema-retry counts and logprob-dip statistics per candidate (the free self-signals of Supply §9), wall-clock and GPU-hours per file by stratum (seeds the Supply §8.10 budget unit), and `bad_anchor` rates specifically (anchor discipline is the cheapest thing an adapter can fix — a candidate failing *only* on anchors is a different situation than one failing on truth).

## 7. Kill criterion 2 — baseline arm

Runs concurrently with §6; shares nothing with it but the repo.

- **Task set**: 20–30 real tasks on the pilot repo, drawn from recent issue/commit history, spanning task classes (bug fix, feature extension, "explain/where-is" questions). Frozen before any Loam artifacts exist.
- **Measurement**: existing paired-run harness records tokens-per-task, split into discovery vs. execution where the harness can tell, plus the three-tier task outcome.
- This is the **baseline arm only**. The treatment arm (same task classes, live bundle, real CLI) runs at step 6 of §2. Matching is by task class, not literal task identity, to dodge memorization effects.

## 8. Deliverables and exit

- `p0/sample_manifest.json`, `p0/labels/`, `p0/models.lock`, per-candidate result tables with exact intervals, baseline-arm token table.
- **P0 Results memo** (one page): per-stratum pass/fail per candidate, the gate decision with the winning model (or the kill/gate rationale), finalized tolerances for P1, and the observed claim-density and GPU-hour numbers that turn the Supply §8.10 budget and §16.6 open question into empirical answers.
- On pass: component TRDs may begin, in the §2 order. On fail: the results memo *is* the kill/gate document.

## 9. Open questions (scoped to this protocol)

1. Ontology v0 for soft claims — the minimal claim-type list this experiment extracts (intent, convention, gotcha, contract?) and one labeled example each, needed before prompt freeze.
2. Pilot repo selection — which in-house repo has all three claim surfaces and a usable recent-task history for §7.
3. Whether the Qwen3.5-4B floor probe is worth its labeling cost if early spot-checks show it far below tolerance (option: score its critical strata only, abandon early with a recorded note).
