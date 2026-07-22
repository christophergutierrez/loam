# Implementation Plan: Execute Loam P0 (model-floor experiment + KC2 baseline → gate decision)

## Planning Verdict
- **verdict:** READY_WITH_ASSUMPTIONS
- **task_tier:** standard
- **tier_trigger:** Multi-component greenfield build (measurement harness, serving, scoring) with contract-ish artifacts (frozen manifest, labels, tolerances, models.lock) and one security-sensitive surface (secret redaction on a real repo). No removal → migration/state-diagram machinery omitted. Re-tier to **full** if Consumption P0 (public CLI surface) or the bundle write path is pulled into scope — both explicitly out of scope here.
- **execution_policy:** cost_optimized
- **model_routing:** current-model-only
- **model_tiers:** current runtime model for fast / standard / reasoning
- **reason:** Phased P0 execution plan grounded in the reconciled PRDs + ADRs 0001–0006. READY_WITH_ASSUMPTIONS (not READY) because (a) greenfield ⇒ most gate baselines are *unrun* until the milestone builds their harness; (b) the cross-family shakedown is an existence-spike that may fail and trigger a thesis replan; (c) the terminal milestone (E1) is an inherent human gate. Pass-3 review (3 independent reviewers) fixed 3 Blocking + 9 Material + 9 Minor; none remain open.

## Repository State (Staleness Contract)
- **HEAD:** `6311a1c8e8e8c20a5b70a55b4d6dd4542f870b16` (branch `main`) — unchanged; all session work is uncommitted.
- **Discovered at:** 2026-07-22T22:13:56Z (re-validated same HEAD at pass-3 review).
- **Dirty set (uncommitted, to preserve):** `.gitignore`, `CLAUDE.md`, `README.md`, `docs/CONSUMPTION_PRD.md`, `docs/GLOSSARY.md`, `docs/P0_EXPERIMENT_PROTOCOL.md`, `docs/SUPPLY_PRD.md` (modified); `docs/P0_EXECUTION_PLAN.md`, `docs/adr/` (untracked — **ADR-0001…0006**, six ADRs). `.sdlc/`/`.killhouse/` are not present/ignored.
- **Executor pre-check:** if any `docs/**` or `docs/adr/**` file changed since this HEAD+dirty state, treat the cited facts as stale and re-validate — they are this plan's spec.

## Repository Findings
- `fact: repo is greenfield (docs-only) <- command: find . -name '*.py' -o -name '*.rs' … -> (empty)`. No source, no build/test commands yet ⇒ most gate baselines are recorded `unrun: greenfield — this milestone builds the harness that makes the gate runnable`.
- `fact: stacks <- .gitignore + ADR-0006 -> Rust (loam-core/CLI, demand), Python (pipeline/experiments, supply)`. P0 execution work (harness, extraction runner, scoring, labeling) is **Python**; Rust `loam-core` is Consumption-P0 (post-gate), out of scope here.
- `fact: p0/ layout pre-anticipated <- .gitignore -> "p0/raw/, p0/runs/ ignored; manifests, labels, models.lock committed"`.
- `fact: spec = docs/ + docs/adr/0001–0006`. This is the CONTEXT_DOCS grounding in lieu of code.
- **Context docs read:** all four docs + all six ADRs.

## Requested Outcomes & Non-Goals
- **O1** — a frozen, stratified sample (manifest + criticality bands + labels + committed tolerances) exists and is immutable post-freeze.
- **O2** — a mechanical verification harness auto-scores mechanical claims deterministically, with language coverage of the pilot repo.
- **O3** — stock T1 + a *cross-family* stock T2 (**different base family**) both serve on the owned backend (the shakedown; ADR-0005). *(spike)*
- **O4** — per-candidate, per-stratum extraction precision measured with exact (Clopper-Pearson) intervals over **human/harness ground-truth labels**, under the ADR-0004 re-weighting (soft primary; anchor/type ≤1%; mechanical yield).
- **O5** — the three-state broken-stratum tripwire (ADR-0004) implemented and applied per stratum.
- **O6** — anchor-claims-per-GPU-hour + claim-density + GPU-hours recorded (ADR-0001 window sizing).
- **O7** — KC2 baseline arm: the rediscovery-ceiling *interval* via the tool-call-class discovery/execution split (ADR-0003).
- **O8** — KC2's ≥30% target sanity-checked against the measured ceiling before any treatment arm.
- **O9** — a P0 Results memo carrying the explicit, evidence-cited gate decision.

**Non-goals:** the `loam-core` CLI / any command; the bundle *write* path; TraceStore intake/flush; any QLoRA adapter (P0 is generation-0 stock); the KC2 *treatment* arm; all P1–P3 loops. Post-gate and speculative to detail now.

## Facts, Assumptions, and Decisions
- **Facts (cited above):** greenfield; Python for P0; spec in docs/.
- **Assumption (low-risk):** the pilot repo exists in-house with all three claim surfaces + usable recent-task history (P0 §9 Q2; A1 resolves).
- **Assumption (low-risk):** the owned backend serves a ~35B sparse-MoE + a cross-family model sequentially (A3 spike proves/refutes).
- **Decision (resolved, ADR-0006):** `loam-core`/CLI = Rust (demand); pipeline = Python (supply); the "never Rust" rule was sdlc-only and does not apply.
- **Decision (resolved, P0 §9 Q3):** the Qwen3.5-4B floor probe is scored **critical strata first**; on a clear critical-stratum failure it is **abandoned early with a recorded note** rather than fully labeled (protects the binding human-labeling budget). Reflected in c2.
- **Decision — human gate (terminal):** the P0 pass/kill/gate decision (E1) is a human call.

## Outcome Traceability Matrix
| outcome_id | outcome | milestone_id(s) | invariant_id(s) | final_check | baseline_verified |
| --- | --- | --- | --- | --- | --- |
| O1 | frozen labeled sample + bands + tolerances | a1, a5 | INV-sample-frozen, INV-labels-immutable, INV-census-only, INV-tolerances-derived | manifest hashes match; labels unedited; tolerances unedited | unrun (greenfield) |
| O2 | mechanical harness + lang coverage | a2 | INV-exact-binomial | harness scores seeded true/false claims; pilot languages covered | unrun (greenfield) |
| O3 | cross-family serving | a3 (spike) | INV-anchor-stock | two **different-family** endpoints serve a real-size prompt | unrun (spike) |
| O4 | per-stratum precision | a4, b1, c1, c2 | INV-exact-binomial, INV-tolerances-derived | per-candidate per-stratum CP intervals + verdicts over human labels | unrun (greenfield) |
| O5 | tripwire | c2 | — | a sized seeded-bad stratum yields BROKEN (CP lower > tol, w/ margin) | unrun (greenfield) |
| O6 | sizing readouts | c3 | — | anchor-claims/GPU-hr recorded, >0 & plausible | unrun (greenfield) |
| O7 | rediscovery ceiling | d1, d2 | — | discovery/execution split reported as interval + band width | unrun (greenfield) |
| O8 | KC2 ceiling check | d3 | — | target ≤ capturable OR decision = revise | unrun (greenfield) |
| O9 | gate decision memo | e1 | all | memo exists; winner consistent with c2 verdicts; decision cites evidence | unrun (human gate) |
| — | secrets never enter any surface | a2, b1, c1 | INV-no-secrets | seeded fixture secret caught pre-redaction; every write surface scans clean | runnable at A2 first step |

_No orphan milestones; every milestone maps to an outcome._

## Final-State Invariants
```yaml
- id: INV-sample-frozen
  statement: Sampled files' content-hashes at scoring time equal the frozen manifest; post-freeze edits drop the file (recorded), never silent-swap.
  category: regression
  check: re-hash each manifest path; diff against p0/sample_manifest.json
  baseline_polarity: "unrun: no manifest yet (a1)"
  post_condition: all hashes match or file recorded as dropped
  scope: phase-end
  cost: cheap
  rationale: O1
- id: INV-labels-immutable
  statement: Labels are committed BEFORE the first committed c2 verdict table, and never edited afterward.
  category: absence
  check: "git log p0/labels/ : (a) last label commit timestamp precedes the commit of p0/runs-derived c2 verdict tables that ARE committed to p0/labels/verdicts/ (define this as the anchor artifact — NOT gitignored p0/runs/); (b) no p0/labels edits after that anchor commit"
  baseline_polarity: "unrun: no labels yet (a5)"
  post_condition: committed-before holds AND no post-anchor edits
  scope: final
  cost: cheap
  rationale: O1 — anti p-hacking. Anchor is a committed artifact, not the gitignored run dir.
- id: INV-census-only
  statement: Every P0 label carries an explicit sample_stream field == census; no random/directed ledger split exists in P0.
  category: presence
  check: "every p0/labels entry HAS a sample_stream field AND it == census; a missing field FAILS (not vacuously passes)"
  baseline_polarity: "unrun: no labels yet"
  post_condition: 100% present-and-census
  scope: phase-end
  cost: cheap
  rationale: O1 — prevents P1 dual-ledger tooling from later mis-reading P0 labels as random-stream and polluting rate estimates.
- id: INV-anchor-stock
  statement: The T2 anchor runs stock (no adapter) throughout P0 (generation 0).
  category: absence
  check: serving config for T2 lists no LoRA/adapter
  baseline_polarity: "unrun: T2 not stood up (a3)"
  post_condition: T2 config adapter-free
  scope: every-pass
  cost: cheap
  rationale: O3 — CLAUDE.md non-negotiable 3
- id: INV-no-secrets
  statement: No secret material (or anything derived from a secret value) enters any surface — INCLUDING the serving-layer prompt/request log. Deterministic redaction runs on SOURCE TEXT before prompt construction (redact → prompt → model), not merely before claim caching.
  category: absence
  check: gitleaks/trufflehog-class scan of EVERY write surface (p0/, claim cache, serving prompt/request logs, temp caches, p0/runs/) returns clean; a seeded fixture secret is redacted to a declarative token before it can reach the prompt or any log
  baseline_polarity: "RUN AT A2 FIRST STEP (see a2): seed a known test secret into a fixture; an off-the-shelf scanner MUST flag it pre-redaction — cite the command + flagged output. Not yet run at plan time (greenfield: no fixture/scanner wired)."
  post_condition: scan clean; seeded secret present only as {{loam:secret ...}} token; serving prompt/request logging disabled-and-confirmed
  scope: every-pass
  cost: cheap
  rationale: CLAUDE.md non-negotiable 7. Redaction must precede prompt construction, or a secret transits the prompt log first.
- id: INV-exact-binomial
  statement: All interval math is exact (Clopper-Pearson / Rule of Three / Beta-binomial); Wald/normal approximation is banned.
  category: absence
  check: "scoring lib unit tests assert BOTH: (a) k=0 case — 95% upper bound ≈ 3/n (Rule of Three); (b) ≥1 small-n k>0 case matching reference Clopper-Pearson bounds to a fixed tolerance. (A single k>0 point cannot distinguish CP from Wald; the k=0 case is the discriminator since Wald degenerates to [0,0].)"
  baseline_polarity: "unrun: no scoring code (b1/c2)"
  post_condition: both assertions pass; no normal-approx present
  scope: final
  cost: cheap
  rationale: O4 — CLAUDE.md non-negotiable 4
- id: INV-tolerances-derived
  statement: Per-stratum tolerances are committed at freeze (p0/tolerances.yaml); sample sizes derive from them; undersupply is fixed by extending the sample (§5 rule 3), never by editing a tolerance.
  category: regression
  check: p0/tolerances.yaml committed at a1; git history shows no post-freeze edits; any sample extension cites §5 rule 3
  baseline_polarity: "unrun: tolerances committed at a1"
  post_condition: tolerances file unchanged post-freeze
  scope: final
  cost: cheap
  rationale: O4 — P0 §5 rule 1
```
**Cheap per-pass subset:** INV-no-secrets, INV-anchor-stock. **Full suite at:** phase-end / final.

## Phased Plan

### Phase A — Foundations (prerequisites; mostly no models)
- **objective:** stand up everything the measurement needs before the tracer bullet. **prerequisites:** none. **blast_radius:** secret redaction on a real repo (a2/INV-no-secrets — the single highest-blast-radius surface). **rollback_boundary:** delete `p0/` scaffolding + harness package. **exit_gate:** a1–a5 pass (or a3 spike stop-condition triggers replan).

#### Milestone: a1-frozen-sample
- **outcome:** `p0/sample_manifest.json` (paths + content-hashes, committed, frozen), **plus** an S0-style **criticality-band assignment** committed alongside it, **plus** `p0/tolerances.yaml`. Bands are computed by *defined deterministic heuristics* (import fan-in centrality, git churn, test coverage, file size, public-contract detection — P0 §3), with the thresholds recorded. Stratified per §5 (critical×soft & standard×soft primary gates; oversample so the anchor/type ≤1% bound has ~300 claims across ~800–1,200 total). traces_to: O1.
- **tracer_bullet:** prerequisite.
- **acceptance_gates:** what is checkable *at freeze* — per-band **file** counts vs the §5 weights (~60 critical / ~40 standard); every manifest entry has path + sha256; band-assignment file + `p0/tolerances.yaml` committed. **Per-stratum *claim* supply adequacy is explicitly deferred to c2** (checked after extraction; extend via §5 rule 3 if short). **baseline_polarity:** manifest/bands/tolerances absent → fails (presence). **evidence_to_record:** per-band file counts vs §5 weights; the heuristic thresholds used.
- **stop_conditions:** no in-house repo has all three claim surfaces + task history → escalate (P0 §9 Q2).

#### Milestone: a2-mechanical-harness
- **outcome:** a deterministic harness (tree-sitter / import analysis / schema introspection) that returns pass/fail for a mechanical claim + anchor, covering **every language in the pilot repo's sampled files** (uncovered languages recorded as a coverage gap for E1, never silently dropped). It wires the **S1 secret-redaction detector to run on source text BEFORE prompt construction** (redact → prompt → model). **First action of this milestone runs the INV-no-secrets failing baseline** (seed a fixture secret; off-the-shelf scanner flags it) and cites it. traces_to: O2. invariants_at_risk: INV-no-secrets, INV-exact-binomial.
- **acceptance_gates:** (1) seeded-fixture baseline run and cited (scanner flags the secret pre-redaction); (2) on a fixture set covering each mechanical claim TYPE (signatures, imports, schema columns, config keys) + a `bad_anchor` case, harness confirms every true claim and rejects every false one; (3) pilot-repo language set enumerated and each confirmed covered; (4) a seeded secret is redacted to a `{{loam:secret …}}` token and never reaches the prompt or any log. **baseline_polarity:** harness absent → fails; seeded secret un-redacted at baseline → INV-no-secrets proven non-vacuous. **evidence_to_record:** scanner command+output; confusion counts; language-coverage list.

#### Milestone: a3-cross-family-shakedown  *(SPIKE — ADR-0005)*
- **outcome:** the discovered fact, with evidence: a stock T1 (sparse-MoE) and a stock T2 that is a **DIFFERENT base family** both serve on the backend; `p0/models.lock` pins exact checkpoints + serving image digests + **base-family metadata for each**. traces_to: O3. invariants_at_risk: INV-anchor-stock.
- **tracer_bullet:** prerequisite.
- **acceptance_gates:** (1) **`family(T1) != family(T2)`** asserted mechanically from `models.lock` metadata — a hard pass/fail, not just recorded ids (a same-family pair FAILS); (2) both endpoints return a **valid completion** — defined as *parses as the frozen extraction schema and is non-empty/non-degenerate* — on a **representative extraction-size prompt** (long prefill: real file + evidence-context recipe, thousands of tokens); (3) the **sequential T1-unload → T2-load cycle** completes within memory/KV. **baseline_polarity:** endpoints not up, OR same-family pair, OR real-size prompt OOMs → fails. **evidence_to_record:** the two families, the two completions, measured VRAM/KV headroom.
- **stop_conditions (MANDATORY):** no viable *cross-family* pair serves → **halt and replan the thesis** — `corroborated` and anchor-independence are unbuildable; this is a kill/gate input.

#### Milestone: a4-ontology-v0-and-prompt-freeze  *(prerequisite of a5 and b1/c1)*
- **outcome:** the soft-claim **ontology v0** (minimal claim-type list — intent / convention / gotcha / contract — one labeled example each, P0 §9 Q1); the generation-0 **extraction prompt frozen** with few-shot + file-alone evidence recipe; **and the decode config frozen** (temperature 0 or fixed-low, single run, no self-consistency voting — P0 §6). traces_to: O4. **tracer_bullet:** prerequisite — a consistent, reproducible extraction target must exist before labeling (a5) or the tracer (b1).
- **acceptance_gates:** ontology v0 + frozen prompt + frozen decode config committed *before* a5 and b1; artifact is content-hashed. **baseline_polarity:** no ontology / unpinned prompt or decode config → extraction target undefined & non-reproducible → fails. **evidence_to_record:** ontology doc + prompt hash + decode params.
- **stop_conditions:** if ontology v0 cannot be pinned without a taxonomy decision, surface it (it changes what every stratum measures).

#### Milestone: a5-labeling-rubric  *(depends on a4)*
- **outcome:** the P0 §4 labeling rubric + two-labeler flow; `p0/labels/` initialized. **The rubric's claim types must match the a4 ontology exactly** (this check lives here). traces_to: O1. invariants_at_risk: INV-labels-immutable, INV-census-only.
- **acceptance_gates:** rubric committed; **rubric claim types == a4 ontology** (mismatch fails); on a pilot subset, adjudicated disagreement < 5% (else fix rubric and relabel the affected subtype before scoring any model); labels carry `sample_stream: census`. **baseline_polarity:** no rubric/labels, or type mismatch → fails. **evidence_to_record:** adjudicated disagreement rate.

### Phase B — Tracer bullet (thin end-to-end measurement on a tiny slice)
- **objective:** prove the *entire* P0 measurement path on ~5 files + one candidate before scaling. **prerequisites:** a1–a5 **and d1 committed** (d1's baseline task set must be frozen before any Loam artifact — including b1 — exists). **rollback_boundary:** discard the tracer outputs. **exit_gate:** b1 gate passes.

#### Milestone: b1-tracer-end-to-end  *(TRACER BULLET)*
- **outcome:** for ONE candidate over a ~5-file frozen subset: extract (from the a4 frozen prompt) → **redact source before prompting** → mechanically auto-score mechanical claims → **run the T2 anchor over the soft claims (exercising the anchor serving layer)** → hand-label the soft claims → compute a per-stratum **Clopper-Pearson** interval over the human labels → emit a mini result table incl. the anchor/type rate, the **anchor recall vs human ground truth** (NOT the §11.2 conditional-miss-rate, which is a P1 same-base-vs-anchor seeded measurement), and anchor-claims-per-GPU-hour. traces_to: O4 (+ exercises O2, O3, O5 scaffolding, O6). tracer_bullet: yes.
- **acceptance_gates:** a result table with an exact interval for ≥1 stratum, produced end-to-end by the harness (serving→extraction→redaction→mechanical→anchor→labeling→exact-binomial→readout) — not hand-assembled — with every layer including T2 anchor serving exercised. **baseline_polarity:** pipeline absent → no table → fails. **evidence_to_record:** the table + the command that produced it.

### Phase C — Full model-floor experiment (scale out)
- **objective:** run candidates over the full sample and produce gate-grade verdicts, **within a fixed GPU-hour budget**. **prerequisites:** b1. **rollback_boundary:** discard `p0/runs/` for a candidate. **exit_gate:** c0–c3 pass.
- **c0 — budget gate (run first):** fix the **P0 GPU-hour budget cap** for the run (seeded from a3's measured anchor/extraction throughput and the Supply §8.10 budget unit) and record a **written early-abandon rule**: large-reference models (e.g. 120B-class) and the 4B floor probe are scored **critical strata first**; on a clear critical-stratum failure or on crossing the budget cap, stop that candidate with a recorded note. **gate:** budget cap value + early-abandon rule committed before c1 scales out. **baseline_polarity:** no cap/rule recorded → fails.

#### Milestone: c1-extraction-full
- **outcome:** stock generation-0 extraction over the full frozen sample for each candidate (from `models.lock`), claims cached with typed anchors, **redaction applied to source before prompting**. Candidate scope honors c0's early-abandon rule. traces_to: O4. invariants_at_risk: INV-no-secrets.
- **acceptance_gates:** for each in-scope candidate × sampled file, claims are cached and the **anchor-resolution *rate*** (path+hash+span contains the quoted evidence) is **recorded** (feeding c2's ≤1% anchor/type bound). *Note: c1 does NOT require 100% anchor resolution — that would make c2's ≤1% bound vacuous; c1 records the rate, c2 gates on it.* **baseline_polarity:** no claims cached → fails. **evidence_to_record:** claim counts + anchor-resolution rate per candidate.

#### Milestone: c2-falsify-score-verdict
- **outcome:** humans label soft claims per rubric; the mechanical harness auto-scores mechanical claims — **these human/harness labels are the ground truth from which per-candidate per-stratum Clopper-Pearson intervals are computed** (the KC1 gate). The T2 anchor is run over soft claims *separately* to produce an **anchor recall vs human ground-truth** readout (the anchor's verdicts never define the extractor's error rate in P0). Apply the **three-state tripwire** to the human/harness-scored intervals; evaluate the ADR-0004 gate (soft 3%/5% primary; anchor/type ≤1% global from c1's recorded rate; mechanical = yield readout). **4B floor probe:** score critical strata first; abandon early on clear critical failure (recorded). traces_to: O4, O5. invariants_at_risk: INV-exact-binomial, INV-tolerances-derived.
- **acceptance_gates:** every in-scope candidate has a per-stratum verdict from exact intervals over human/harness labels; **a seeded-bad stratum with a specified injected error fraction and n chosen so the Clopper-Pearson *lower* bound exceeds tolerance with margin yields BROKEN** (proves the tripwire non-vacuous and deterministic — an under-sized injection landing UNPROVEN does not count); undersupplied strata extended per §5 rule 3, never by relaxing tolerance. **baseline_polarity:** no verdicts → fails; sized seeded-bad stratum NOT BROKEN → tripwire vacuous (Blocking). **evidence_to_record:** per-candidate per-stratum interval + verdict tables (committed to `p0/labels/verdicts/` as the INV-labels-immutable anchor); the anchor-recall table.

#### Milestone: c3-sizing-readouts
- **outcome:** record the promoted readouts — anchor-claims-per-GPU-hour (ADR-0001), bad_anchor+bad_type rate (the ≤1% gate), mechanical verification-survival rate, schema-retry/logprob self-signals, GPU-hours per file by stratum. traces_to: O6.
- **acceptance_gates:** all readouts present AND **anchor-claims-per-GPU-hour passes a sanity bound (>0 and within a plausible order of magnitude for the backend)** — not a bare presence check, since a zero/garbage value silently mis-sizes P1's census/sampling regime. **baseline_polarity:** absent or non-sane → fails. **evidence_to_record:** the readout tables + the commands.

### Phase D — KC2 baseline arm (parallel track; no extraction models)
- **objective:** measure the rediscovery ceiling on real tasks. **prerequisites:** a1 (the selected pilot repo) + a **per-call-token-instrumented paired-run harness** (d2's prereq) — *not* a2/a3/a4/a5. **d1 must complete before b1** (Phase B prereq) so the baseline task set is frozen before any Loam artifact exists. **rollback_boundary:** discard baseline outputs. **exit_gate:** d1–d3 pass.

#### Milestone: d1-frozen-task-set
- **outcome:** 20–30 real tasks from recent issue/commit history, spanning classes (bug fix / feature / explain-where-is), frozen & committed **before any Loam artifact (including b1) exists**. traces_to: O7.
- **acceptance_gates:** committed task set, timestamped before b1; **count ∈ [20,30]**; **≥1 task per required class**. **baseline_polarity:** absent, out-of-range count, or a missing class → fails. **evidence_to_record:** task list + class tags + freeze commit.

#### Milestone: d2-baseline-run-and-ceiling
- **outcome:** paired-run harness records tokens-per-task + three-tier outcome; the discovery/execution split is computed at **tool-call-class** granularity with a conservative/liberal band → the rediscovery-ceiling **interval** (ADR-0003). **Prereq:** the harness must emit **per-call token counts** — if it emits only totals, add that instrumentation first. traces_to: O7.
- **acceptance_gates:** a baseline token table + a rediscovery-ceiling interval exist, **with the band width reported** (so a uselessly-wide ceiling is visible as low-information). **baseline_polarity:** harness lacks per-call tokens → split not computable → fails. **evidence_to_record:** the interval + band width + tagging rule.

#### Milestone: d3-kc2-ceiling-check
- **outcome:** compare KC2's ≥30% target against the measured ceiling; decide keep/revise **before** any treatment arm. traces_to: O8.
- **acceptance_gates:** a recorded comparison; **conditional — if the ≥30%-of-total target exceeds what the d2 band says is capturable, the decision MUST be "revise" (with written rationale); a "keep" is valid only when target ≤ capturable.** **baseline_polarity:** no comparison, or a "keep" that violates the condition → fails. **evidence_to_record:** the comparison + decision + rationale.

### Phase E — Gate decision (human gate)
- **objective:** the P0 verdict. **prerequisites:** c0–c3, d1–d3. **exit_gate:** e1.

#### Milestone: e1-results-memo-and-decision  *(MANDATORY HUMAN GATE)*
- **outcome:** the P0 Results memo (P0 §8): per-stratum pass/fail per candidate (soft gates + anchor/type; mechanical yield); the **decision** — smallest candidate passing *all* soft strata + the anchor/type bound wins T1; only 120B-class passes → **gate**; no local candidate passes → **KC1 fired, kill/gate on a revised thesis**; the cross-family shakedown result; the rediscovery-ceiling interval + KC2 check; finalized tolerances for P1; anchor-claims/GPU-hr + claim-density + GPU-hours. traces_to: O9.
- **acceptance_gates:** memo committed AND (1) **a mechanical consistency sub-check: the declared winner's c2 verdicts are all PASS on the required soft strata + the anchor/type bound (no UNPROVEN/BROKEN stratum passed off as a win)**; (2) the decision cites its evidence (c2 verdict tables, a3 shakedown result, d2 ceiling, d3 check). **The human ratifies the kill/gate framing, not the arithmetic — the arithmetic is checked mechanically.** **baseline_polarity:** no memo, or winner inconsistent with verdicts → fails. **stop_conditions:** human call — surface tables + recommended verdict, then wait; do not auto-advance to any post-gate phase.

### Phase F+ — Post-gate (CONTINGENT on E1 = PASS; intentionally not detailed)
Replan each via the PLAN loop on E1 PASS:
- **F — Supply P0 remainder:** OKF bundle schema; stock-pipeline run over the pilot repo → a real bundle (P0 §2 step 4).
- **G — Consumption P0:** `loam-core` (Rust, per ADR-0006) + `get`/`search`/`bundle`/`observe`, corpus resolution, stanzas, **spool-only** telemetry (ADR-0002).
- **H — KC2 treatment arm:** paired runs vs the live bundle (P0 §2 step 6).
- **P1–P3** (PRD §Phasing): gate; compounding; consumption loops — each gated on the prior.

## Consolidated Verification
P0 is complete when: (1) frozen sample + bands + tolerances + immutable labels exist (INV-sample-frozen/labels-immutable/census-only/tolerances-derived); (2) every in-scope candidate has per-stratum exact-interval verdicts over human labels under ADR-0004, with a *sized* seeded-bad stratum proving the tripwire non-vacuous; (3) the anchor ran stock and **verifiably cross-family** (INV-anchor-stock, a3 family check); (4) no secret entered any surface incl. prompt logs, redaction ran before prompting (INV-no-secrets — baseline run at a2); (5) all math exact, discriminated at k=0 (INV-exact-binomial); (6) the rediscovery-ceiling interval + a condition-enforced KC2 check exist; (7) the Results memo carries a winner mechanically consistent with the c2 verdicts, human-ratified.

## Replan Triggers
- a3 finds no cross-family pair → thesis replan (corroborated unbuildable).
- Any critical stratum returns BROKEN for *every* candidate → KC1-adjacent; feeds E1 kill/gate.
- An undersupplied stratum can't be extended (§5 rule 3) → report achievable bound; E1 calls it out.
- Budget cap (c0) crossed → invoke the early-abandon rule; recorded.
- Measured rediscovery ceiling ≪ assumed 60–80% → revise KC2 target (d3) before any treatment arm.
- Any `docs/**` spec file changed since the recorded HEAD+dirty state → re-validate cited facts.

## Review Record
- **Pass 1 — draft:** Lead Planner. Standard tier; greenfield ⇒ gate baselines recorded `unrun` with reasons.
- **Pass 2 — inline gate audit** (the independent subagent died on an API error; degraded to inline per PLAN sec 13): 7 findings applied (precision-source, secret-scan scope, shakedown realism, language coverage, missing ontology milestone, e1 evidence, tracer-anchor, band width).
- **Pass 3 — three independent reviewers (subagents, completed; `gate_audit: independent`):** GateQuality/Falsifiability, Completeness/Sequencing/ColdStart, Risk/Alignment/Simplification. 3 Blocking + 9 Material + 9 Minor — **all accepted (some adapted); no lens conflicts.** Dispositions:
  - **[Blocking] a4↔a5 ontology cycle** (labeling gated before the ontology it needs) → **reordered: a4 = ontology+prompt+decode freeze, a5 = labeling; the "types match ontology" check now lives in a5.**
  - **[Blocking] cross-family never verified** → a3 now asserts `family(T1) != family(T2)` mechanically from models.lock (hard pass/fail); "valid completion" defined.
  - **[Blocking] redaction ordered after prompt** → redaction moved to run on **source text before prompt construction**; INV-no-secrets covers the serving prompt/request log; logging-disabled is a post_condition.
  - **[Material] missing criticality-band triage** → a1 gains a defined-heuristic band-assignment sub-step, committed as evidence.
  - **[Material] a1 claim-supply unverifiable at freeze / non-executable jq** → a1 gate rewritten to per-band *file* counts; claim-supply adequacy deferred to c2 + §5 rule 3.
  - **[Material] Phase B prereq omitted a5 / d1** → Phase B prereq = a1–a5 **and d1 committed**.
  - **[Material] budget cap not operationalized** → new **c0 budget gate** (cap value + written early-abandon rule) before c1.
  - **[Material] 4B early-abandon unmapped (§9 Q3)** → resolved as an explicit Decision + encoded in c0/c2.
  - **[Material] c1 100%-anchor contradicts c2 ≤1%** → c1 records the anchor-resolution *rate*, does not require 100%.
  - **[Material] INV-no-secrets baseline asserted-not-run** → the failing baseline is now the **first action of a2**, run and cited; the plan no longer claims it proven.
  - **[Material] d3 decision unconstrained by ceiling** → d3 gate made conditional (target > capturable ⇒ MUST revise).
  - **[Material] e1 winner not checked vs verdicts** → e1 gains a mechanical winner-consistency sub-check; human ratifies framing only.
  - **[Material] INV-exact-binomial weak discriminator** → check now requires the k=0 (3/n) case + reference CP points.
  - **[Material] INV-labels-immutable anchor undefined** (results gitignored) → anchor defined as committed `p0/labels/verdicts/` tables; checks committed-before + no-edit-after.
  - **[Material×2] conditional-miss-rate miscited (§11.2)** → renamed to "anchor recall vs human ground truth"; §11.2 cite dropped (it's a P1 same-base-vs-anchor seeded metric).
  - **[Minor] decode config not frozen** → added to a4 freeze.
  - **[Minor] Phase D prereq over-broad** → narrowed to a1 repo + token-harness.
  - **[Minor] INV-tolerances-derived no artifact** → committed `p0/tolerances.yaml` at a1.
  - **[Minor] INV-census-only near-vacuous / field-absence** → strengthened (field must be present AND census; absent = fail) + stated the P1-tooling failure it prevents.
  - **[Minor] a3 "valid completion" undefined** → defined (parses frozen schema, non-degenerate).
  - **[Minor] d1 count/classes unchecked** → gate asserts count ∈ [20,30] + ≥1 per class.
  - **[Minor] c2 seeded injection margin unspecified** → gate specifies injected fraction + n so CP lower > tol with margin.
  - **[Minor] c3 readouts presence-only** → anchor-claims/GPU-hr sanity bound (>0, plausible).
  - **[Minor] ADR inventory stale (0005/five)** → Repository State updated to 0001–0006 / six.
- **Conflict-triage:** none — findings independent, no lens conflicts.
- **Convergence:** pass 3 resolved 21, introduced 0, regressed 0. No Blocking or Material remains open. Verdict READY_WITH_ASSUMPTIONS (greenfield unrun baselines + a3 spike + e1 human gate).

```json
{
  "verdict": "READY_WITH_ASSUMPTIONS",
  "task_tier": "standard",
  "tier_trigger": "multi-component greenfield build with one security-sensitive surface (secret redaction on a real repo); no removal",
  "execution_policy": "cost_optimized",
  "model_routing": "current-model-only",
  "model_tiers": { "fast": "current-model", "standard": "current-model", "reasoning": "current-model" },
  "passes": 3,
  "open_blocking_findings": 0,
  "open_material_findings": 0,
  "vacuous_gates_found": 0,
  "cold_start_gaps": 0,
  "uncited_facts": 0,
  "gate_audit": "independent",
  "staleness": { "head": "6311a1c8e8e8c20a5b70a55b4d6dd4542f870b16", "dirty_files": ["CLAUDE.md","README.md","docs/CONSUMPTION_PRD.md","docs/GLOSSARY.md","docs/P0_EXPERIMENT_PROTOCOL.md","docs/SUPPLY_PRD.md",".gitignore","docs/P0_EXECUTION_PLAN.md","docs/adr/(0001-0006)"], "discovered_at": "2026-07-22T22:13:56Z" },
  "traceability_complete": true,
  "orphan_milestones": [],
  "characterization_gaps": [],
  "conflicts_resolved": [],
  "cheap_every_pass_invariants": ["INV-no-secrets","INV-anchor-stock"],
  "blast_radius_decisions": ["P0 gate pass/kill/gate decision (E1) is a human call","cross-family shakedown (a3) is an existence spike; failure = thesis replan","secret redaction on a real pilot repo — redact-before-prompt + prompt-logging disabled (INV-no-secrets)"],
  "human_decisions_required": ["E1 gate decision"],
  "plan_location": "docs/P0_EXECUTION_PLAN.md",
  "summary": "Phased P0 execution plan (A foundations: sample+bands+tolerances, mechanical harness+redaction-before-prompt, cross-family shakedown w/ family check, ontology+prompt+decode freeze, labeling -> B tracer bullet -> C full model-floor under a budget cap -> D KC2 baseline -> E human gate). Falsifiable gates with baselines; ADR-0001..0006 honored; post-gate phases contingent. Pass-3 independent review fixed 3 Blocking + 9 Material + 9 Minor; none open."
}
```
