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
  - **claimed** — single-source, quarantine-marked; consumers are told to verify before load-bearing use.
  - **Ceiling rule:** same-base endorsement can never raise a claim above `claimed`. The T1.5 falsifier is purely subtractive — it kills claims, it elevates nothing.
- **Evidence anchor.** See overloads above. Typed: source path + content hash of the span + line span + quoted evidence. No anchor, no admission.
- **Anchor index.** Derived SQLite index mapping anchor spans → concept IDs, rebuilt at S4, invalidated with the bundle, read by `loam get`/`search`/`lint`. Rebuildable state, never a source of truth.
- **Provenance.** Frontmatter record of extractor version, endorsing falsifiers, and resolution path (`mechanical | cross_family | cloud | human | none`).
- **Redaction token.** Declarative replacement for a detected secret: `{{loam:secret type=... rule=... ref=...}}`. Never a mask or plausible default (those re-trigger secret scanners). Nothing derived from the secret value — not even a hash — is stored.
- **Stale / STALE.** A concept whose evidence anchors no longer hash-match current source. Returned to readers *with* a STALE marker, never silently withheld or silently served.
- **Gardening.** Background re-extraction/re-verification of the content-hash dirty set; the steady-state pipeline run.

## Verification and sampling

- **Falsifier.** Any model whose job is to kill wrong claims. Two kinds, never interchangeable: the *T1.5 falsifier* (same-base adapter; cheap filter, subtractive only) and the *anchor model* (T2, cross-family, stock; the decorrelation reference and the only LLM path to `corroborated`).
- **Census.** 100% falsification coverage of a stratum. Mandatory for the `critical` criticality band.
- **Stratum.** A sampling cell: file type × criticality band × adapter route (extensible with induced features). Tolerances and sample sizes are per-stratum.
- **Tolerance.** The configured per-stratum maximum acceptable error rate — the primitive from which sample size is derived (not the other way around).
- **Rule of Three.** Zero-failure exact bound: with n clean samples, the 95% upper bound on the error rate is ≈ 3/n. 1% tolerance → 300 clean; 0.5% → 600; 3% → 100. Wald/normal intervals are banned in this regime.
- **Dual ledgers.** The two sampling streams whose denominators must never mix: **random stream** (unbiased measurement; feeds dashboards and kill criteria) and **directed stream** (targeted hunting; feeds quarantine and cleanup). The `sample_stream` frontmatter field enforces the separation.
- **Pursuit (directed pursuit).** On a falsification hit, sampling more in the direction of the induced failure predicate, with SPRT early stopping and GPU-hour caps.
- **Failure predicate.** A queryable, induced description of a failure cluster ("adapter v3 fails on YAML anchors/aliases"). Feeds pursuit, the weakness registry, hard-negative curriculum, and seeded-mutation templates.
- **Weakness registry.** The live list of open failure predicates, consulted by S0 triage to route matching files to heavier extraction until cleared.
- **Unexplained bucket.** Failures that refuse to cluster under current coordinates. Its growth rate is the feature-space-debt health metric.
- **Feature-space debt.** Accumulated evidence that error clusters exist in coordinates the system cannot yet see.
- **Seeded mutation.** A known-bad claim injected into falsification batches to measure live gate recall (Killhouse pattern). Falsifiers are scored on kills against seeded ground truth, never on agreement.
- **Conditional miss rate.** Of mutations the extractor plausibly produces, the fraction a given falsifier misses — measured separately for same-base vs. anchor to quantify the correlation penalty.

## Consumption

- **Task bundle.** See overloads above. Output of `loam bundle`: index match + explicit link traversal, size-capped, trust tiers surfaced. Dumb-first by decision (Consumption §5.3).
- **Inbox.** The queue `loam observe` writes to. Untrusted input: hints about where to look, never evidence. The pipeline re-derives all evidence from source. This is the prompt-injection boundary and the reason no consuming harness needs to be trusted.
- **Observation.** A typed inbox entry: `claim`, `contradiction`, `concept-wrong`, `concept-missing`, `procedural`.
- **Write boundary.** Consuming agents never write concepts; agents propose, the pipeline disposes.
- **Consultation rate.** Fraction of agent sessions on an ingested corpus that consult Loam before source exploration, per harness. Kill criterion; the instruction layer's report card.
- **Bundle fallback rate.** Fraction of consumed task bundles followed by immediate fallback exploration for the same question. (Formerly misnamed "bundle hit rate" — it bounds *misses*.) Kill criterion.
- **Read-then-grep.** An agent consults a concept, then verifies against source anyway. Expected for `claimed`; a gap or distrust signal for `verified`.
- **Miss event.** A zero-result `loam search` — the concrete sensor for demand-driven extraction; clustered miss shapes reveal ontology gaps.
- **Telemetry spool.** Local append-only SQLite queue for telemetry events; async flush to TraceStore; never blocks a command, never silently drops. Spool-only operation is a supported degraded mode.
- **Stanza.** The per-harness instruction block (CLAUDE.md / AGENTS.md / Skill) shipped with the bundle; a generated artifact, refreshed by `loam init --refresh`.
- **Heat.** TraceStore-derived read frequency. Measures exposure-if-wrong; legitimately promotes criticality up to census.

## Models and hardware

- **Workhorse.** The single adapter-carrying base (Qwen3.6-35B-A3B): extraction, T1.5 falsification, and the linter's semantic tier via multi-LoRA.
- **Anchor model.** See overloads above. Stock cross-family falsifier (GPT-OSS-120B or Nemotron 3 Nano). Stays stock; tuning it erodes the decorrelation it exists to provide.
- **Adapter generation.** A numbered QLoRA training round. Promotion requires the seeded-mutation audit gate plus the stock-vs-tuned regression sample (which is a regression suite, not a verification tier).
- **GB10 / DGX Spark.** The single target box: 128GB unified memory, ~273 GB/s bandwidth, sm_121. Bandwidth-bound decode favors sparse-MoE and long-prefill/short-output workloads.
- **Qwen3.5 vs. Qwen3.6.** Intentional split, not a typo: Qwen3.6 ships no small dense variant, so triage (T0) uses the Qwen3.5 dense line while extraction (T1) uses Qwen3.6-35B-A3B. Tokenizer compatibility across the lines is to-be-verified (Supply §16.7).
