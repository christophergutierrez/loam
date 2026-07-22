# Loam — Supply PRD v0.1

**Local extraction, verified knowledge, compounding agents.**

| | |
|---|---|
| Status | Draft v0.1 |
| Owner | Chris Gutierrez |
| Reference platform | Sunk-cost owned hardware, reached through a uniform endpoint abstraction (ADR-0005). Reference config (not a requirement): DGX Spark (GB10, 128GB unified, sm_121, ~273 GB/s); equally H200s, a large Mac, or a proxy fronting any of these |
| Date | 2026-07-22 |

---

## 1. Problem Statement

Coding and knowledge agents spend the majority of their tokens on **rediscovery**: grepping, reading, and rebuilding a mental model of a corpus that was fully understood by a previous session and then discarded. The working hypothesis is that discovery overhead is 60–80% of task tokens; **P0 measures this rather than assuming it** (ADR-0003) — the baseline arm's discovery-vs-execution token split is the rediscovery share and therefore the ceiling on achievable savings. Either way the resulting mental model evaporates at session end.

Local inference changes the economics of the obvious fix. Exhaustively processing every file in a corpus is prohibitive with metered cloud models but is a fixed-cost background job on owned hardware. The binding constraint is therefore **not compute — it is trustworthiness of what gets stored**. A wrong fact retrieved into an agent's context is worse than no fact: the agent trusts it and stops verifying. Extraction errors from small local models fail silently and compound.

Loam is a knowledge store plus an extraction/verification pipeline whose central design commitment is that **the write path is the product**. Retrieval format, storage engine, and agent integration are all downstream of one question: can we admit knowledge at a known, measured, bounded error rate using local models, escalating to paid models only for a small, criticality-gated fraction?

### 1.1 Why now

- Google's Open Knowledge Format (OKF v0.1, published June 12, 2026) formalized the LLM-wiki pattern (markdown + YAML frontmatter, explicit links) into a portable standard, giving Loam a storage layer that is agent-native, greppable, diffable, and git-resident.
- Sparse-MoE open models (e.g. Qwen3.6-35B-A3B, GPT-OSS-120B, Nemotron 3 Nano) fit the bandwidth-bound decode profile of sunk-cost boxes, making exhaustive multi-pass extraction feasible on owned hardware. Model names are examples; tiers are defined by required properties and P0 selects the checkpoints (§5, ADR-0005).
- Existing in-house components slot in directly: TraceStore (provenance, content-addressed blobs, demand signals), Killhouse (falsifiable gates, seeded-mutation audits), the skill evaluation framework (paired-run token benchmarking), and the Amesh LoRA training pipeline (weight-baking distillation target).

## 2. Goals and Non-Goals

**Goals**

1. Reduce tokens-per-task for agents operating on ingested corpora by a measurable margin (see §12 kill criteria) with no task-quality regression.
2. Admit knowledge to the store at a measured, per-stratum-bounded error rate, using local models for ≥99% of inference volume.
3. Make the store self-maintaining: content-hash staleness invalidation, incremental re-gardening, and use-time trust promotion/demotion.
4. Make the verification gate itself auditable: the system continuously measures its own false-accept rate via seeded mutations.
5. Compound: verified failures become adapter training curriculum; verified knowledge becomes skills, context bundles, and eventually LoRA-baked weights.

**Non-Goals (v1)**

- Multi-user / team sync. Loam v1 is single-operator, single-Spark.
- Real-time ingestion latency. The pipeline is batch-phased by design.
- General document management or search UX for humans. Humans interact via git and the conflict/weakness queues only.
- Cloud-hosted operation. Cloud models appear only as the Tier-3 escalation endpoint.

## 3. Corpus and Prioritization

**Corpus definition.** "Anything that helps an agent do better work": source code, configs, schemas, internal docs, design notes, and (P2, §15) TraceStore traces. The corpus is unbounded by intent, therefore **prioritization is a first-class design axis**, not an optimization.

**Demand-driven processing.** First agent contact queues a file for deep extraction in the next batch pass — contact raises priority; it does not create a real-time latency guarantee (see §2 non-goals) — and files are gardened in the background thereafter. Exhaustive crawling is reserved for explicitly onboarded corpora. TraceStore access frequency is the canonical heat signal: process the hot set deeply; leave the cold set as raw source until touched.

**Onboarding a corpus** = one full cold-start pipeline run (§6). Steady state = the identical pipeline run on the content-hash dirty set. One pipeline, two corpus sizes.

## 4. Storage Layer

**Format: OKF-shaped bundle.** A directory of markdown concept documents with YAML frontmatter and explicit markdown cross-links, resident in git alongside (or mirroring) the source it describes. Relationships are explicit links an agent follows, not embedding similarity it guesses at.

**Why not the alternatives (recorded for posterity):**

| Option | Verdict | Reason |
|---|---|---|
| Vector RAG | Entry-point index only, optional | Lossy, weak multi-hop, chunks strip structure; fine for fuzzy recall |
| Knowledge graph / GraphRAG | Rejected as canonical layer | LLM triple extraction is the precision bottleneck; incremental invalidation is painful; the code graph is computable deterministically (tree-sitter, import analysis) and should be |
| KAG (OpenSPG) | Rejected | Heavyweight; benefits only if query patterns are demonstrably relational |
| CAG (prefix-cache) | Adopted as accelerator | Local-only advantage: precompute KV of the small hot stable core on owned vLLM; not a store |
| OKF / LLM-wiki | **Canonical layer** | Agent-native, explicit links, greppable, diffable, human-auditable, git staleness for free; agents excel at wiki bookkeeping (Karpathy) |

**Frontmatter schema (per concept), minimum fields:**

```yaml
concept_id: <stable id>
sources:                      # every claim anchors to source spans
  - path: <repo-relative path>
    content_hash: <sha256 of source span>
    span: [start_line, end_line]
trust_tier: verified | corroborated | claimed
claim_type: mechanical | soft   # mechanically checkable vs. intent/convention
provenance:
  extractor: <model+adapter version>
  falsifiers: [<model versions that endorsed>]
  resolution: <mechanical | cross_family | cloud | human | none>
sample_stream: random | directed | census | unsampled
extracted_at: <timestamp>
last_validated: <timestamp>
```

**Derived anchor index.** The pipeline's S4 write stage also (re)builds a derived SQLite index mapping anchor spans → concept IDs. It is read by `loam get` (live anchor verification), `loam search`, and `loam lint` (mechanical tier), and is invalidated and rebuilt together with the bundle it derives from. It is rebuildable state, never a source of truth — the markdown bundle remains canonical.

**Conflict objects are first-class.** When sources genuinely disagree (doc contradicts code, two docs contradict each other), the disagreement is stored as a conflict concept and surfaced to the human/repair queue. Source conflicts are among the highest-value outputs of the system and are never discarded as extraction noise.

## 5. Model and Adapter Topology

One **workhorse base** carries all adapters; one **anchor base** stays stock. Sequential load/unload per pipeline stage (each stage gets the box's full bandwidth and full KV headroom); co-residency is not required by any stage.

| Tier | Model | Adapters | Role |
|---|---|---|---|
| T0 triage | Qwen3.5-2B or 4B (dense) | triage QLoRA (gen ≥1) | Routing metadata only — **never claims** |
| T1 extraction | Qwen3.6-35B-A3B (NVFP4, MTP on) | extraction QLoRA; code-extraction route (Qwen3-Coder-30B-A3B optional) | Claim generation with evidence anchors |
| T1.5 cheap falsifier | same base, multi-LoRA | falsifier QLoRA (entailment-trained) | Near-free first-pass kill of sloppy errors |
| T2 anchor falsifier | GPT-OSS-120B MXFP4 **or** Nemotron 3 Nano (stock) | **none — anchor stays stock** | Cross-family falsification; the decorrelation reference |
| T3 escalation | Cloud frontier model | n/a | Only on (disagreement × criticality); rate is a monitored health metric |

*Naming notes.* (1) Model tiers (T0–T3) and pipeline stages (S0–S4, §6) are distinct namespaces — S2 runs the T1.5 falsifier; S4 runs no model at all. (2) Triage deliberately uses the Qwen**3.5** dense line while extraction uses Qwen**3.6**-35B-A3B: Qwen3.6 ships no small dense variant. Any design that assumes a shared tokenizer (no re-tokenization between triage and extraction) therefore rests on Qwen3.5↔3.6 tokenizer compatibility, a to-be-verified fact (§16.7). (3) Names in this table are model families; the P0 protocol and serving recipes pin exact HF checkpoint IDs and vLLM image digests — family names are not deployable identifiers. (4) **Tiers are defined by required properties, not checkpoints, and the backend is agnostic** (ADR-0005): T0 small dense; T1 sparse-MoE, adapter-carrying, fits the box's KV headroom; T2 *different base family from T1*, stock, strong entailment. Hardware is likewise reference-config, not dictated. P0's model-floor experiment is the selection mechanism, and whether a cross-family pair *stands up at all* on the chosen backend is an explicit P0 shakedown gate. (5) **Census-by-default (ADR-0001) makes the anchor falsify every soft claim, so T2 is selected for throughput, not raw strength** — favoring a lighter/faster resident cross-family model (e.g. Nemotron-class) with an optional heavier nightly pass (e.g. 120B-class).

**Design rules:**

1. **Anchor independence is sacred.** The anchor is the reference electrode; tuning it on pipeline-derived data erodes exactly the decorrelation it exists to provide. If ever tuned, only on cloud-verified hard negatives, followed by a mandatory seeded-mutation re-audit.
2. **Shared-base falsification is a filter, not a gate.** Same-base endorsement (tuned-vs-stock or extractor-vs-T1.5-falsifier) can never raise a claim above *claimed* — the T1.5 falsifier is purely subtractive: it kills claims, it elevates nothing. Shared priors mean shared blind spots, and LoRA cannot drift far enough from the base to escape them; `corroborated` requires cross-family agreement by definition (§7.3).
3. **Error diversity ranking** (spend accordingly): different base family > different evidence context (file alone vs. file+callers vs. file+docs) > different task (generation vs. entailment) > different training mix > different seed (≈ worthless).
4. **Adapter economics.** Adapters bake the extraction ontology, instructions, and few-shot examples into weights, deleting 2–4K tokens of system prompt per file pass, and raise schema adherence. Multi-LoRA hot-swap makes the adapter library ~free at serve time.
5. **Sizing floor is empirical.** Day-one experiment: run 4B, 35B-A3B, and one large model over the same labeled 100-file sample; the smallest model whose precision clears the stratum tolerance wins its tier. Cheap extraction that fails falsification is not cheap.
6. Pin exact vLLM image tags/digests per model per recipe (backend-specific flags — e.g. sm_121 on a GB10 — are model-specific; copying flags across models or backends silently regresses).

## 6. Pipeline (cold start ≡ steady state)

Phased batches with sequential model loading. Steady-state input is the content-hash dirty set; onboarding input is the full corpus. Stages are numbered **S0–S4**; model tiers keep the T0–T3 names of §5 (distinct namespaces).

**Pre-flight sizing (before S0, per corpus).** A deterministic estimate — scannable-file count (corpus-inclusion filter applied) × per-file token estimate × measured backend throughput, compared against the gardening window (~8h provisional) — decides the **regime for anchor coverage of soft claims** (ADR-0001): *census regime* (falsify every soft claim) when full census fits, else *sampling regime* (the §8 apparatus rations coverage within the window's budget, criticality×heat first). Mechanical claims are verified deterministically and exhaustively in both regimes; the fork is soft-claim coverage only. The estimate is sharp only after P0 supplies claim-density and anchor-claims-per-GPU-hour numbers; a rough sizer before then.

- **S0 — Triage sweep** (runs the T0 model). Small model + deterministic heuristics over every file. Deterministic first: import fan-in centrality, git churn, test coverage, file size, public-contract detection → criticality score. LLM only for what heuristics cannot judge (authoritative doc vs. scratch note). Output: routing metadata (file class, complexity, criticality, adapter route, evidence-context recipe, priority). **S0 emits no claims** — low-precision claims would waste compute or, worse, anchor later passes.
- **S1 — Extraction.** T1 workhorse + extraction adapter (stock + full prompt on generation 0). Long-prefill/short-output profile — the favorable regime for a bandwidth-bound sunk-cost box. Claims cached with typed evidence anchors and content hashes. Extractor self-signals (logprob dips, schema-retry counts) logged from pass one; they are free and later serve as clustering coordinates (§9).
- **S2 — Cheap falsification** (once the T1.5 falsifier adapter exists). Same-base entailment sweep over cached claims; kills sloppy errors at near-zero marginal cost.
- **S3 — Anchor falsification.** Unload workhorse, load the T2 anchor, falsify surviving soft claims by the regime chosen at pre-flight (ADR-0001): **census by default** (every soft claim), or — when census won't fit the window — the §8 sampling design (budget-filling, criticality×heat first). 100% of critical is always census; directed pursuit on hits. Mechanical claims are already verified deterministically (§7.2) and do not consume the anchor budget.
- **Resolution path.** Disagreement → mechanical check where the claim type allows (tree-sitter, import analysis, schema introspection — zero tokens) → cloud on disagreement×criticality → human queue for genuine source conflicts.
- **S4 — Write + compound.** No model resident. Resolved claims write to the store with trust tier and full provenance. Qualifying resolutions join the adapter training pool (§10). Weakness registry and unexplained bucket update (§9).

**Throughput framing.** First-contact ingestion of a large repo is an overnight batch, priced once. Thereafter a typical commit dirties a handful of files; incremental gardening is minutes of background work; agents read the wiki at near-zero marginal cost. The honest pitch: *pay one slow pass per corpus, then coast.*

## 7. Evidence Contract

Every claim admitted to the store satisfies:

1. **Typed anchor.** Source path + content hash + line span + quoted evidence for every claim. No anchor, no admission.
2. **Mechanical-first verification.** Claims about code structure (signatures, imports, schema columns, config keys) are verified deterministically, not by LLM, at ~zero token cost. The LLM verification budget is reserved for *soft* claims (intent, conventions, gotchas, rationale) — a much smaller surface, and the one where cross-model disagreement is most informative.
3. **Trust tiers with teeth.** `verified` (mechanically checked) > `corroborated` (cross-family agreement + evidence-span entailment, survived falsifier) > `claimed` (single-source; quarantine-marked in frontmatter; consuming agents are told).
4. **Use-time promotion/demotion.** When an agent uses a `claimed` fact in a real task and the work confirms it → promotion signal; contradiction → demotion + re-extraction queue. Normal agent work is free verification labor; the store's trust distribution improves with use instead of rotting.
5. **Provenance restriction on training data.** Only claims verified mechanically or via cross-family/cloud corroboration may enter adapter training pools. Claims whose only endorsement is same-base agreement are ineligible (self-training collapse guard, §10).
6. **Redaction Contract (declarative, upstream, versioned).** Secrets must never enter any Loam surface — wiki, claim cache, telemetry, falsification batches, or adapter training pools (QLoRA memorization of a repeated secret is a leak that survives every rotation).
   - **Detection is deterministic and runs at S1 (extraction), before claims are cached**: gitleaks/trufflehog-class rule packs plus entropy heuristics, zero LLM. S4 write-time redaction alone is insufficient — intermediate caches leak too.
   - **Replacement is declarative, never a mask or plausible default.** Masks (`AKIAXXXX…`) and fake defaults still match secret-scanner regexes, turning the wiki into a permanent DLP false-positive source. Instead a structured token with no exploitable format: `{{loam:secret type=aws_access_key_id rule=v3.2 ref=r-7f3a}}`. This preserves the legitimate knowledge ("an AWS key is configured here") while storing nothing of the value.
   - **Registry, not values.** Each redaction logs type, location, and rule version to a registry for audit. Nothing derived from the secret value is stored — not even hashes (hashes of low-entropy secrets are crackable).
   - **Versioned rules with re-redaction sweeps** on rule updates (TraceStore versioned-redaction pattern), so improved detectors retroactively clean earlier extractions.

## 8. Sampling Design (Falsification Budget Allocation)

**Regime first (ADR-0001).** Cross-family falsification of soft claims is **census by default** — because local compute is a sunk cost, the anchor falsifies *every* soft claim. This section (sampling) is the **fallback** the pre-flight estimate (§6) selects only when full census won't fit the gardening window; there, coverage is *budget-filling* (spend the whole window, criticality×heat first — never a fixed 1% floor), the overflow tail stays `claimed` and drains across successive gardening passes, and the apparatus below also serves its always-on roles: measuring extractor precision (kill criteria) and discovering failure clusters (§9). 100% of `critical` is census in either regime.

Method (sampling regime): **zero-acceptance stratified sampling with adaptive (Thompson-style) cluster pursuit**, denominated in GPU-hours.

1. **Strata** = useful distinctions: file type × criticality band × adapter route, extensible to induced features (§9). Criticality band `critical` → census (100%).
2. **Tolerance-derived minimums (exact binomial — never normal approximation).** Per-stratum error tolerance is the configured primitive; sample size derives from it via the exact zero-failure bound: P(0 errors in n) = (1−p)ⁿ, i.e., the Rule of Three (95% upper bound ≈ 3/n on a clean sample). 1% tolerance → 300 clean; 0.5% → 600; 3% → 100. The Wald/normal interval is banned: it degenerates to [0,0] at zero observed errors and its np≳5 validity condition fails exactly in the rare-error regime this system lives in.
3. **Interval reporting**: Clopper-Pearson (or Jeffreys). **Cross-batch accumulation**: per-stratum Beta-binomial posterior — evidence compounds across pipeline runs without repeated-peeking inflation; priors are tunable (new adapter route starts skeptical; long-clean stratum starts trusting).
3a. **Three-state broken-stratum tripwire (ADR-0004).** Read the same Clopper-Pearson interval from *both* ends: **PASS** = upper bound ≤ tolerance (proven clean); **UNPROVEN** = interval straddles tolerance (keep sampling — the §8.8 escalation path); **BROKEN** = *lower* bound > tolerance (proven to exceed it — the extractor has left the rare-error regime). BROKEN fires an alarm, quarantines that stratum (claims withheld from promotion, failure cluster routed to the weakness registry + human queue), and *stops* sampling that stratum — more samples do not fix a broken extractor. It quarantines the offending stratum only; healthy strata proceed. Self-scaling: a single stray error just widens the interval (no alarm), while *several* errors trip it.
4. **Census cutoff.** If the derived sample exceeds ~0.5 of the stratum population, sample the whole stratum (finite population correction makes sampling gains collapse there anyway).
5. **Dual ledgers — never mix denominators.** `random` stream → unbiased rate estimation; drives health dashboards and kill criteria. `directed` stream → detection and cleanup; drives quarantine and re-extraction. Pooling directed samples into rate estimates biases the estimate (you looked where errors live); one schema field (`sample_stream`) prevents the corruption forever.
6. **Directed pursuit.** On a hit: induce the failure predicate (§9), sample more *in that direction* — files matching the predicate — while maintaining (and up-weighting) random samples within the stratum. Sequential probability ratio test permits early stop on strong evidence either way.
7. **Stopping rules for pursuit** (all three armed): directed hit rate returns to stratum background; cluster exhausted; per-incident GPU-hour cap trips (remainder flagged to next batch cycle).
8. **Adaptive escalation.** A stratum whose posterior disagreement rate crosses threshold auto-escalates sampling (up to census) until it recovers. Configured n% is a floor for healthy strata, not a cap.
9. **Event-triggered elevation.** Every adapter promotion and every corpus onboarding runs elevated sampling for the first N files — the moments the error distribution most likely shifted.
10. **Budget cap.** All of the above allocates within a per-run GPU-hour budget (files are the wrong unit: a 200K-token file+callers bundle costs ~50× a 4K-token file).

**What falsification does, by regime (explicit, so nobody misreads it later).** Under **census-by-default (ADR-0001), cross-family falsification *is* the protection** for soft claims: every soft claim is anchor-falsified, so surviving soft claims reach `corroborated` and the admitted-error surface is bounded by anchor recall, not by a sample. The "measurement, not protection" caveat applies **only in the sampling-regime fallback** (huge repos where census won't fit the window): there the random sample is *measurement* — it bounds the extractor's error rate per stratum at constant cost regardless of corpus size — while *protection* for the unfalsified overflow comes from the census on critical strata, trust-tier quarantine (`claimed` = not-yet-corroborated, draining across gardening passes), and use-time promotion. In both regimes the random sample's job is to detect an unhealthy extractor (via the §8.3a tripwire); the tiers' job is to contain damage while it is fixed.

## 9. Failure Predicates, Weakness Registry, and Feature-Space Debt

**Failure predicates are first-class artifacts.** On clustered failures, an induction pass over failure artifacts (claim + source + falsifier rationale + extractor self-signals) emits a queryable predicate, e.g. *"extraction adapter v3 fails on YAML with anchors/aliases."* Each predicate compounds in four places:

1. Directs pursuit sampling (§8.6).
2. Registers in the **weakness registry**, consulted by S0 triage: matching files route to heavier extraction or straight to the anchor until the entry is cleared. Clearing an entry (predicate re-sampled clean after retrain) is the cleanest evidence a retrain fixed something.
3. Becomes hard-negative curriculum for the next adapter generation (§10).
4. Becomes a seeded-mutation template for gate audits (§11) — mutations shaped like *observed* failure modes, not invented ones.

**Feature space is constructed, not given (the soil disanalogy).** Contamination clusters by physics; extraction errors cluster by shared cause, and cause-space has no God-given metric. Two files adjacent in every obvious feature can fail for unrelated reasons; the real cluster may be visible only in a coordinate nobody defined yet.

- **Missing-coordinate detector.** Directed sampling that fails to concentrate (pursuit hit rate ≈ background) means the cluster doesn't exist *or the current coordinates can't see it*. Failures that will not cluster go to an explicit **unexplained bucket**; its growth rate is a monitored health metric (feature-space debt).
- **Abductive coordinate proposal.** Unlike soil, every failure ships with artifacts. When the unexplained bucket accumulates, an LLM pass hypothesizes the common cause ("these all involve re-exported symbols"); the hypothesis becomes a candidate feature; pursuit sampling tests it. The coordinate system is learnable and the system proposes its own coordinates.
- **Seed coordinates**: static metadata (type, size, import shape, directory) + extractor self-signals (logprob dips, schema retries, falsifier disagreement scores). "The model found this hard" is often a better proximity metric than anything computable from the file alone, and it is logged free from pass one.

## 10. Adapter Training Loop

Batch process with a gate — never continuous per-discrepancy updates.

1. **Pool**: provenance-restricted training candidates (§7.5) + weakness-registry hard negatives. The falsifier adapter's adversarial curriculum is self-generating: every extractor error caught by anchor or cloud is a labeled hard negative on precisely the shared-prior blind spots — the one mechanism that pushes same-base decorrelation *up* over time (the Red Queen loop, running in training rather than only at inference).
2. **Train** generation N+1 (QLoRA on GB10 as background job). *Open risk*: PEFT target-module support for Qwen3.6's Gated DeltaNet hybrid layers is unverified — confirm before committing to this base (same failure class as the PEFT→MLX incompatibility).
3. **Audit gate before promotion**: seeded-mutation audit (§11) + fixed regression sample (a few hundred files rerun stock-vs-tuned to catch adapter-induced drift — this comparison is a regression suite, **not** a verification tier; same-base agreement measures what the adapter changed, not what is true).
4. **Promote or reject.** Promotion triggers event-elevated sampling (§8.9).
5. **Known cost of tuning, monitored**: fine-tuning makes errors as fluent and schema-perfect as successes; the informative stumble-signals of a base model winging it get trained away. The falsifier tier must work harder as the extractor improves — tracked via §11 conditional miss rates per generation.

## 11. Adversarial Gate Audit

The verification gate is itself under test, continuously.

1. **Seeded mutations.** Known-bad claims injected into falsification batches; templates drawn from the weakness registry (observed failure shapes) plus synthetic classes. A falsifier that never finds errors is either facing a great extractor or is lazy; seeding tells the difference and yields live gate recall.
2. **Conditional miss rates.** Of mutations the extractor plausibly produces, measure the fraction missed by (a) same-base falsifier adapter vs. (b) cross-family anchor. This directly quantifies the correlation penalty and sets tier trust weights from data instead of priors.
3. **Retrain-triggered re-audit.** Mandatory after every adapter promotion (extractor or falsifier) and any anchor change. The feedback-collapse failure mode presents as same-base kill rate improving on *old* error classes while silently degrading on whatever the new generation invents — only a refreshed seeded audit sees it.
4. **Falsifier scoring.** Falsifiers are scored on kills against seeded ground truth (Killhouse pattern), never on agreement.

## 12. Metrics and Kill Criteria

**Kill criteria (falsifiable; any failure kills or gates the project). Numeric targets are provisional until the P0 Experiment Protocol fixes per-stratum tolerances — the criteria's form is the commitment:**

1. **Extraction precision**: on a hand-labeled ~100-file day-one sample, best local extractor precision clears the per-stratum tolerances of §8.2, **re-weighted per ADR-0004**: the gate rests on **soft-claim precision** (no production backstop) plus a tight **global anchor/type-correctness bound (≤1%)**; mechanical *truth* precision is relaxed to a yield/competence readout (a wrong mechanical claim fails deterministic verification and is never admitted, so it governs yield, not admitted-fact trust). If no local model clears the soft + anchor/type bar, the sunk-cost thesis fails.
2. **Token economics**: ≥30% tokens-per-task reduction on paired runs (existing harness) against a matched task set, with no task-quality regression — **sanity-checked against the measured rediscovery ceiling (ADR-0003)** before the treatment arm runs: a 30%-of-total target must not exceed what the discovery/execution split says is capturable, or the target is revised rather than pursued.
3. **Staleness bound**: after one week of normal repo churn, ≤2% of retrieved concepts contradicted by current source.
4. **Gate recall**: falsifier tier catches ≥90% of seeded mutations, measured per §11, sustained across adapter generations.

**Health metrics (dashboards, Beta-binomial posteriors per stratum):** per-stratum error posteriors (random stream only); cloud escalation rate (creep = extractor degradation or corpus character shift); unexplained-bucket growth (feature-space debt); conditional miss-rate gap (same-base vs anchor); weakness-registry open/cleared counts; trust-tier distribution drift; promotion/demotion rates from agent use; GPU-hours per run vs. budget.

## 13. Agent Consumption (Token-Reduction Mechanisms)

Ascending ambition; the wiki is the intermediate representation feeding all downstream forms:

1. **Lookup replaces exploration** — the agent reads three concept pages instead of twenty greps. ~80% of expected savings; trivially benchmarkable.
2. **Task-scoped context compilation** — a router assembles the linked-concept bundle for a task instead of naive top-k chunks; trust tiers are surfaced to the consumer.
3. **Distillation** — recurring *procedural* knowledge graduates to skills (existing seven-metric eval decides admission); declarative facts stay in the wiki.
4. **Weight-baking** — high-confidence stable knowledge trains LoRA adapters via the Amesh pipeline: zero context cost, slow cycle, hard invalidation; reserved for the most stable core.
5. **CAG acceleration** — precomputed prefix KV cache of the hot stable core on owned vLLM; a local-only economic advantage.

## 14. Staleness Contract

1. Every claim carries source content hashes (§7.1). A cheap deterministic pass over any change set flags stale concepts — no LLM in the invalidation path.
2. Stale concepts enter the re-garden queue; the steady-state pipeline (§6) is the cold-start pipeline applied to the dirty set.
3. The wiki lives in git; updates ride post-commit gardening passes, making knowledge freshness observable in diffs.
4. `last_validated` timestamps + kill criterion 3 bound corpus-wide staleness; LoRA-baked knowledge (hardest to invalidate) is restricted to the slowest-moving verified core.
5. Trust demotion from agent contradiction (§7.4) catches what hash invalidation cannot (semantic staleness with unchanged bytes elsewhere).

## 15. Phasing

- **P0 — Prove the thesis (gate: kill criteria 1).** Labeled 100-file sample; model floor experiment (§5.5); mechanical verification harness; OKF bundle schema; T1/T2 stock-model pipeline on one repo; paired-run token benchmark, **baseline arm only** — pre-Loam token counts on matched tasks, which needs no wiki and no consumption layer; the treatment arm lands with Consumption P0 (dependency ordering: P0 Experiment Protocol).
- **P1 — The gate.** Sampling engine (§8) with dual ledgers and exact-binomial math; seeded-mutation audit; conflict objects; trust tiers; dirty-set incremental runs.
- **P2 — Compounding.** Extraction + triage + falsifier adapters with the §10 loop; weakness registry; unexplained bucket + abductive coordinate proposal; T1.5 multi-LoRA serving; TraceStore trace ingestion into the corpus (§3).
- **P3 — Consumption.** Context-bundle router; use-time promotion/demotion wiring into agent workflows; skill-graduation pipeline; CAG hot core; (stretch) LoRA weight-baking of the verified core.

## 16. Open Questions

1. PEFT/QLoRA target-module support for Qwen3.6 Gated DeltaNet layers (blocker check for §10; fallback base: Qwen3-30B-A3B-Instruct-2507).
2. Anchor choice under memory/strength trade: GPT-OSS-120B (stronger, ~60GB, tight KV) vs. Nemotron 3 Nano (~17GB, 80GB+ KV headroom) vs. Nano-resident + nightly 120B heavy-falsification phase. **Census-by-default (ADR-0001) tilts this toward throughput** — the anchor now falsifies every soft claim — favoring the Nano-resident + nightly-120B option; final choice is a P0 selection criterion.
3. Soft-claim ontology v0: which claim types beyond mechanical (intent, convention, gotcha, rationale, contract) and their per-type tolerances.
4. TraceStore integration surface for heat signals and use-time promotion events (schema addition vs. sidecar).
5. Human queue UX for conflict objects and weakness-registry review (git-based? TUI?).
6. Default per-stratum tolerances and the GPU-hour budget for the nightly run (empirical after P0). **Anchor-claims-per-GPU-hour is promoted to a P0 primary readout (ADR-0001)**: it sizes the gardening window and calibrates the §6 pre-flight census/sampling estimator.
7. Qwen3.5↔Qwen3.6 tokenizer compatibility — verify before any design relies on a shared tokenizer (no re-tokenization) between triage (S0) and extraction (S1); the two stages deliberately use different model lines (§5 naming notes).
