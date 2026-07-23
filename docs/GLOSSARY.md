# Loam Glossary

Canonical definitions. Where a term is overloaded elsewhere in the industry (or was overloaded in earlier drafts of these docs), the entry says so explicitly. If a PRD, TRD, or code comment uses one of these words to mean something else, the PRD/TRD/code is wrong, not the glossary — fix it there.

## Deliberately disambiguated overloads

These four pairs are the ones that have already caused (or nearly caused) drift. Read them first.

- **Model tier (T0–T3) vs. pipeline stage (S0–S4).** *Tiers* name models and their roles (Supply §5): T0 triage model, T1 workhorse, T1.5 same-base falsifier adapter, T2 cross-family anchor, T3 cloud escalation. *Stages* name pipeline steps (Supply §6): S0 triage sweep, S1 extraction, S2 cheap falsification, S3 anchor falsification, S4 write + compound. The namespaces are distinct: S2 *runs* the T1.5 falsifier; S4 runs no model at all.
- **Evidence anchor vs. anchor model.** An *evidence anchor* is the typed source reference every claim carries (path + content hash + line span + quote). The *anchor model* (or just "the anchor") is the stock cross-family T2 falsifier. Context usually disambiguates; when it doesn't, write "evidence anchor" or "anchor model" in full.
- **OKF bundle vs. task bundle.** The *OKF bundle* is the store itself: the directory of markdown concept documents in git. A *task bundle* is the output of `loam bundle <task-desc>`: an assembled, size-capped set of concepts for one task. "The bundle" unqualified means the OKF bundle.
- **P-numbers vs. S-numbers.** P0–P3 are roadmap phases (both PRDs §Phasing). S0–S4 are pipeline stages. Never reuse either prefix for anything else.

## Store and claims

- **Claim.** A single extracted assertion about the corpus, carrying at least one evidence anchor. The atomic unit of admission, falsification, and trust.
- **Claim type.** `mechanical` (checkable deterministically: signatures, imports, schema columns, config keys) or `soft` (intent, convention, gotcha, rationale, contract). Mechanical claims are verified by tools, never by LLM vote; the LLM falsification budget exists for soft claims.
- **Concept.** One markdown document in the OKF bundle: a coherent unit of knowledge built from one or more claims, with YAML frontmatter (trust tier, provenance, anchors).
- **Conflict object.** A first-class concept recording that sources genuinely disagree (doc vs. code, doc vs. doc). Never discarded as noise; routed to the human queue. Among the highest-value outputs of the system.
- **Trust tier.** `verified` > `corroborated` > `claimed`.
  - **verified** — mechanically checked against source by deterministic tooling.
  - **corroborated** — cross-family agreement (anchor model endorsement) plus evidence-span entailment, having survived the falsifier chain. Cross-family is definitional: no amount of same-base agreement produces `corroborated`.
  - **claimed** — single-source, not yet cross-family corroborated. A **transient queue state, not a permanent tier**: under census-by-default (ADR-0001) the anchor model drains `claimed` toward `corroborated`, hot/critical-first, across gardening passes. Consumers are still told to verify before load-bearing use. On a huge, actively-churning corpus a cold, non-critical tail may lag indefinitely *in practice*, but `claimed` is never *designed* as permanent quarantine.
  - **Ceiling rule:** same-base endorsement can never raise a claim above `claimed`. The T1.5 falsifier is purely subtractive — it kills claims, it elevates nothing.
- **Evidence anchor.** See overloads above. Typed: source path + content hash of the span + line span + quoted evidence. No anchor, no admission.
- **Anchor index.** Derived SQLite index mapping anchor spans → concept IDs, rebuilt at S4, invalidated with the bundle, read by `loam get`/`search`/`lint`. Rebuildable state, never a source of truth.
- **Provenance.** Frontmatter record of extractor version, endorsing falsifiers, and resolution path (`mechanical | cross_family | cloud | human | none`).
- **Redaction token.** Declarative replacement for a detected secret: `{{loam:secret type=... rule=... ref=...}}`. Never a mask or plausible default (those re-trigger secret scanners). Nothing derived from the secret value — not even a hash — is stored.
- **Stale / STALE.** A concept whose evidence anchors no longer hash-match current source. Returned to readers *with* a STALE marker, never silently withheld or silently served.
- **Gardening.** Background re-extraction/re-verification of the content-hash dirty set; the steady-state pipeline run.

## Verification and sampling

- **Falsifier.** Any model whose job is to kill wrong claims. Two kinds, never interchangeable: the *T1.5 falsifier* (same-base adapter; cheap filter, subtractive only) and the *anchor model* (T2, cross-family, stock; the decorrelation reference and the only LLM path to `corroborated`).
- **Census regime vs. sampling regime.** The two operating modes for anchor (T2) coverage of soft claims, chosen per corpus by the pre-flight feasibility estimate (ADR-0001). **Census regime** (default; small-to-moderate corpora): the anchor falsifies *every* soft claim — no sampling. **Sampling regime** (fallback; corpora whose full-census estimate exceeds the gardening window): the anchor spends the whole window's GPU-hour budget on falsification prioritized by criticality × heat; the Supply §8 stratified/adaptive apparatus rations that coverage. Sampling is a per-window rate limit, not a permanent ceiling — the overflow tail drains across successive gardening passes. Mechanical claims are verified deterministically and exhaustively in *both* regimes; the fork concerns soft claims only.
- **Pre-flight feasibility estimate.** The deterministic sizing pass run before extraction: scannable-file count (corpus-inclusion filter applied) × per-file token estimate × measured GB10 throughput, compared against the gardening window (~8h provisional). Decides census regime vs. sampling regime. Sharp only after P0 supplies real claim-density and anchor-claims-per-GPU-hour numbers; a rough sizer before then.
- **Census.** 100% falsification coverage of a stratum. Mandatory for the `critical` criticality band.
- **Stratum.** A sampling cell: file type × criticality band × adapter route (extensible with induced features). Tolerances and sample sizes are per-stratum.
- **Tolerance.** The configured per-stratum maximum acceptable error rate — the primitive from which sample size is derived (not the other way around).
- **Rule of Three.** Zero-failure exact bound: with n clean samples, the 95% upper bound on the error rate is ≈ 3/n. 1% tolerance → 300 clean; 0.5% → 600; 3% → 100. Wald/normal intervals are banned in this regime.
- **Broken-stratum tripwire.** A three-state per-stratum verdict read from the *same* exact-binomial interval, both ends: **PASS** (Clopper-Pearson *upper* bound ≤ tolerance — proven clean), **UNPROVEN** (interval straddles tolerance — keep sampling; where §8.8 adaptive escalation lives), **BROKEN** (Clopper-Pearson *lower* bound > tolerance — proven to *exceed* tolerance, not merely unproven-good). BROKEN fires an alarm, quarantines that stratum (claims withheld from promotion, failure cluster routed to weakness registry + human queue), and *stops* sampling — a broken extractor is not fixed by more samples. It quarantines the offending stratum only; healthy strata proceed. It marks the moment the extractor has left the rare-error regime the whole sampling design assumes. Self-scaling: 3 errors in 5 samples trips it, 3 in 5000 does not.
- **Dual ledgers.** The two sampling streams whose denominators must never mix: **random stream** (unbiased measurement; feeds dashboards and kill criteria) and **directed stream** (targeted hunting; feeds quarantine and cleanup). The `sample_stream` frontmatter field enforces the separation.
- **Pursuit (directed pursuit).** On a falsification hit, sampling more in the direction of the induced failure predicate, with SPRT early stopping and GPU-hour caps.
- **Failure predicate.** A queryable, induced description of a failure cluster ("adapter v3 fails on YAML anchors/aliases"). Feeds pursuit, the weakness registry, hard-negative curriculum, and seeded-mutation templates.
- **Weakness registry.** The live list of open failure predicates, consulted by S0 triage to route matching files to heavier extraction until cleared.
- **Unexplained bucket.** Failures that refuse to cluster under current coordinates. Its growth rate is the feature-space-debt health metric.
- **Feature-space debt.** Accumulated evidence that error clusters exist in coordinates the system cannot yet see.
- **Seeded mutation.** A known-bad claim injected into falsification batches to measure live gate recall. Falsifiers are scored on kills against seeded ground truth, never on agreement.
- **Conditional miss rate.** Of mutations the extractor plausibly produces, the fraction a given falsifier misses — measured separately for same-base vs. anchor to quantify the correlation penalty.

## Consumption

- **Task bundle.** See overloads above. Output of `loam bundle`: index match + explicit link traversal, size-capped, trust tiers surfaced. Dumb-first by decision (Consumption §5.3).
- **Inbox.** The queue `loam observe` writes to. Untrusted input: hints about where to look, never evidence. The pipeline re-derives all evidence from source. This is the prompt-injection boundary and the reason no consuming harness needs to be trusted.
- **Observation.** A typed inbox entry: `claim`, `contradiction`, `concept-wrong`, `concept-missing`, `procedural`.
- **Write boundary.** Consuming agents never write concepts; agents propose, the pipeline disposes.
- **Consultation rate.** Fraction of agent sessions on an ingested corpus that consult Loam before source exploration, per harness. Kill criterion; the instruction layer's report card.
- **Bundle fallback rate.** Fraction of consumed task bundles followed by immediate fallback exploration for the same question. (Formerly misnamed "bundle hit rate" — it bounds *misses*.) Kill criterion.
- **Read-then-grep.** An agent consults a concept, then verifies against source anyway. Expected for `claimed`; a gap or distrust signal for `verified`.
- **Rediscovery share (discovery/execution split).** The fraction of baseline task tokens spent on *discovery* (grep/read/model-building) vs. *execution* (edits/tests/builds). It is the **ceiling** on achievable savings — distinct from KC2's *achieved* reduction. Measured at tool-call-class granularity with a conservative/liberal band (reported as an interval, not a point). A P0 baseline-arm primary readout (ADR-0003); the founding "60–80%" figure is the hypothesis this measures, not an asserted fact.
- **Miss event.** A zero-result `loam search` — the concrete sensor for demand-driven extraction; clustered miss shapes reveal ontology gaps.
- **Telemetry spool.** Local append-only SQLite queue for telemetry events; async flush to a downstream telemetry consumer; never blocks a command, never silently drops. Spool-only operation is a supported degraded mode.
- **Stanza.** The per-harness instruction block (CLAUDE.md / AGENTS.md / Skill) shipped with the bundle; a generated artifact, refreshed by `loam init --refresh`.
- **Downstream telemetry consumer.** The separate, external system of record that drains the telemetry spool and serves agent traces plus Loam events back for querying. Loam emits; the consumer records; the pipeline queries. Named generically because it is out-of-repo and swappable — v1 depends on none of what it powers (ADR-0002).
- **Heat.** Read frequency reported by the downstream telemetry consumer. Measures exposure-if-wrong; legitimately promotes criticality up to census. A **P1+ amplifier** (ADR-0002): v1 depends on it for nothing — without heat, criticality degrades to static-only (S0 heuristics), and in census regime heat is irrelevant anyway.

## Models and hardware

- **Workhorse.** The single adapter-carrying base carrying extraction, T1.5 falsification, and the linter's semantic tier via multi-LoRA. Defined by *properties* — sparse-MoE, adapter-carryable, fits the backend's KV headroom — not by a fixed checkpoint (e.g., Qwen3.6-35B-A3B); P0's model-floor experiment selects it (ADR-0005).
- **Anchor model.** See overloads above. Stock cross-family falsifier (a *different base family* from the workhorse; e.g., GPT-OSS-120B or Nemotron 3 Nano). Stays stock; tuning it erodes the decorrelation it exists to provide. Under census-by-default (ADR-0001) it falsifies every soft claim, so it is selected for **throughput** (lighter/faster cross-family model resident, optional heavier nightly pass), not raw strength.
- **Adapter generation.** A numbered QLoRA training round. Promotion requires the seeded-mutation audit gate plus the stock-vs-tuned regression sample (which is a regression suite, not a verification tier).
- **Inference backend (endpoint abstraction).** The uniform (OpenAI-compatible-style) interface the pipeline uses to reach every model tier, so no stage is coupled to a specific server, checkpoint, or box (ADR-0005). Motivated by proxies fronting sunk-cost hardware. The intended deployment is sunk-cost local inference on owned hardware; a metered cloud endpoint is a trivial incidental, not the design target.
- **GB10 / DGX Spark.** The *reference* box (not a requirement, ADR-0005): 128GB unified memory, ~273 GB/s bandwidth, sm_121. An example of the sunk-cost owned hardware Loam intends; owned hardware may equally be H200s, a large Mac, etc. Bandwidth-bound decode favors sparse-MoE and long-prefill/short-output workloads.
- **Qwen3.5 vs. Qwen3.6.** Intentional split, not a typo: Qwen3.6 ships no small dense variant, so triage (T0) uses the Qwen3.5 dense line while extraction (T1) uses Qwen3.6-35B-A3B. Tokenizer compatibility across the lines is to-be-verified (Supply §16.7).
