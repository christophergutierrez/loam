# Loam — Consumption PRD v0.1

**How heterogeneous agents find, trust, use, and improve the knowledge store.**

| | |
|---|---|
| Status | Draft v0.1 |
| Owner | Chris Gutierrez |
| Companion to | Supply PRD (supply side: extraction, verification, storage) |
| Date | 2026-07-22 |

---

## 1. Problem Statement

Loam's supply side (Supply PRD) builds a knowledge store whose admission gate bounds error rates. That store is worthless if agents don't consult it, and dangerous if agents can write to it around the gate. The consumption problem has one constraint that shapes every decision here: **the consumers are heterogeneous and not ours.** Work happens in Claude Code, Codex, Grok Build, Antigravity, Hermes, and whatever ships next quarter — different tool models, different instruction conventions, different context disciplines. A bespoke integration per harness is unmaintainable; the integration surface must be tiered by universality, instrumented for compliance measurement, and strictly read-only with respect to the store's admission gate.

The second half of the problem is the feedback loop. Consumption telemetry is the sensor layer the supply-side design assumed but did not implement: use-time trust promotion/demotion (Supply §7.4), demand-driven extraction (Supply §3), heat-based criticality, and live token economics all require knowing when, how, and for what agents used the store. That telemetry is **emitted to a downstream telemetry consumer, which remains a separate product** — Loam produces events; the consumer records them; the pipeline queries them. No recording system is built here.

## 2. Goals and Non-Goals

**Goals**

1. Any agent that can read files can consume Loam with zero setup; any agent that can run bash can consume it with full instrumentation.
2. Consultation is measurable per harness, so instruction-layer failures are diagnosable with data.
3. Every consumption signal the supply side needs (heat, misses, distrust, contradictions, outcomes) is emitted as a typed event.
4. Consumers can propose knowledge and report defects, but **cannot write concepts** — agents propose, the pipeline disposes.
5. Contradictions introduced during work (by humans or agents) are surfaced at edit time, when resolution is cheapest.

**Non-Goals (v1)**

- Building a telemetry store (the downstream consumer's job; Loam emits, full stop).
- Retrieval ranking / vector search in `bundle` (dumb-first; the telemetry earns the ranking layer, see §5.3).
- MCP server (deferred, data-gated; see §3 Tier C).
- Blocking enforcement of lint (advisory only in v1; CI/CD enforcement is explicit future scope, §8.6).
- Human-facing browsing UX (humans use git, their editor, and the queues).

## 3. Integration Tiers

**Tier A — raw files (zero integration).** The OKF bundle is markdown in the repo. Every agent that reads files can consume it with no setup. Entry point: an index concept at the bundle root that agents naturally encounter. Cost: zero. Limitation: unobserved (no telemetry), unrouted (no bundle assembly), undisciplined (agents may over-read). Tier A is the floor, never the target.

**Tier B — the CLI (the workhorse).** Every harness on the target list can run bash, making a small CLI the most universal *instrumented* surface that exists. All engineering investment lands here. The CLI is a thin shell over a client library (`loam-core`) so future surfaces (MCP, editor plugins) wrap the same logic. All output supports `--json`. Every operation emits telemetry as a side effect — instrumentation is not optional per call.

**Tier C — MCP server (deferred, data-gated).** MCP's genuine advantage over the CLI is native tool discovery: Loam's operations appear in the agent's tool list rather than depending on the agent reading and honoring an instruction stanza. That is a difference in *triggering probability*, not capability, and it carries costs the CLI doesn't: per-harness configuration, server lifecycle, and tool schemas occupying context in every session whether used or not. **Decision rule:** build the CLI, measure per-harness consultation rates (§5.2, §9.1); if a harness's compliance stays low after instruction iteration, wrap `loam-core` in a thin MCP server for that harness. Justified by data, not architecture aesthetics.

## 4. CLI Surface

Five commands. Anything that can't be expressed in these five is a design smell to be argued about, not silently added.

| Command | Function | Key behavior |
|---|---|---|
| `loam get <concept>` | Retrieve a concept | Live anchor verification (§7.2): content hashes checked at read time; STALE flag inline if anchors changed since `last_validated`. Trust tier always displayed. |
| `loam search <terms>` | Find concepts | Text/frontmatter search over the bundle. A zero-result search emits a **miss event** — the demand-driven extraction sensor. |
| `loam bundle <task-desc>` | Assemble task-scoped context | Dumb-first (§5.3): index + explicit link traversal from matched entry concepts, size-capped, trust tiers surfaced. Emits bundle composition for outcome joins. |
| `loam observe <report>` | Propose knowledge / report defects | Writes to the **inbox** (§6.1), never to concepts. Typed: `claim`, `contradiction`, `concept-wrong`, `concept-missing`, `procedural`. |
| `loam lint [--staged\|--paths]` | Contradiction check on working changes | Two-tier check (§8). Advisory; never blocks. |

### 4.1 Corpus resolution

Deterministic answer to "which bundle am I reading?": nearest-ancestor discovery (walk up from CWD to the first `loam/` bundle root, `.git`-style), overridable by a workspace config file listing multiple bundles with precedence for cross-repo work. Every telemetry event carries the resolved bundle ID. Unspecified resolution means every harness invents its own behavior; this section exists so they don't.

### 4.2 Read-time staleness

The nightly pipeline bounds corpus-wide staleness (Supply §14), but an agent can read a concept whose anchors were dirtied minutes ago. Because anchors are content hashes, `loam get` verifies them against the working tree in microseconds. Stale reads return the concept **with** a prominent STALE marker and the changed-anchor list — the agent decides whether to trust, verify, or fall back to source. Stale-read events are telemetry (they measure how often the freshness window matters in practice).

## 5. Instruction Layer

### 5.1 Per-harness conventions

Loam ships instruction stanzas as part of the bundle: a CLAUDE.md / AGENTS.md block, a proper Claude Code Skill, and equivalents for Hermes and others as adopted. Core content of every stanza: (1) before exploring source, run `loam bundle` with the task description; (2) trust-tier semantics — `verified` may be relied on, `corroborated` may be relied on with attribution, `claimed` must be verified before load-bearing use; (3) file findings with `loam observe`; (4) run `loam lint` before declaring work done; (5) STALE means check the source.

### 5.2 Compliance is measurable

The instruction layer is the weakest link and is treated as an empirical system: telemetry distinguishes sessions that consulted Loam before exploration from sessions that grepped anyway. **Wiki-consultation rate per harness** is a first-class health metric. Low compliance in a harness is an instruction-tuning problem iterated with data — and, past a threshold of stubbornness, the trigger for that harness's Tier C wrapper (§3).

### 5.3 Dumb-first bundling (decision)

`bundle` v1 does index-match plus explicit link traversal with a size cap — no vector ranking, no learned retrieval. Rationale: agents are good at following links; the failure telemetry (bundles that missed the needed concept, evidenced by post-bundle exploration) identifies exactly where navigation breaks and therefore what a ranking layer must fix. The optional vector index from Supply §4 is earned by that data, not presumed.

## 6. The Write Boundary and the Inbox

### 6.1 Agents propose; the pipeline disposes

**Consuming agents must not write concepts.** The supply-side thesis is that the write path is the product; letting any harness author wiki pages routes unverified single-model output around the entire gate. Instead: `loam observe` files typed observations into an **inbox queue** that enters the normal pipeline as high-priority extraction / re-verification targets, carrying the observing harness, task ID, and evidence as provenance. This also dissolves the multi-harness trust question — Grok Build's judgment never needs to be trusted, because its observations are extraction hints, not admissions.

### 6.2 Inbox is untrusted input

Observations are arbitrary text from arbitrary harnesses operating on arbitrary instructions. The pipeline treats inbox content as **hints about where to look, never as evidence in itself**: an observation's quoted "evidence" is not an anchor; the pipeline re-derives evidence from source. This is the prompt-injection boundary — a malicious or confused observation can waste a re-verification cycle, but cannot admit a claim.

### 6.3 Procedural observations (decision)

The inbox accepts `procedural` observations ("this build command is the one that works") but v1 only tags and parks them in a separate queue. They are future intake for the skill-distillation pipeline (Supply §13.3); coupling the two systems is deferred until the declarative loop is proven. Nothing is lost — the queue accumulates either way.

## 7. Telemetry and Feedback Loops

### 7.1 Event schema (emitted; stored by the downstream telemetry consumer)

Minimum event types, each carrying harness ID, task ID, bundle ID, timestamp: `concept_read` (+ trust tier, stale flag), `search_miss` (+ query shape), `bundle_assembled` (+ composition), `post_bundle_exploration` (agent grepped/read source after consuming a bundle — inferable from trace adjacency), `observation_filed` (+ type), `lint_shown` / `lint_outcome` (§8.5), `task_outcome` (join key to the consumer's existing three-tier outcome signals).

**Durability (decided, not deferred).** Events append to a local SQLite spool and flush to the downstream telemetry consumer asynchronously. Emission never blocks or fails a CLI command, and undelivered events are never silently dropped — the spool persists until acknowledged flush, and spool depth/age is itself a health metric. A CLI that hangs when the consumer is down gets uninstalled; a CLI that silently drops events corrupts every feedback loop in §7.2. Spool-only operation (no downstream consumer reachable) is a supported degraded mode, which also lets Consumption P0 proceed before the intake surface (Supply §16.4) is settled. **v1 depends on no external telemetry consumer (ADR-0002): the feedback loops in §7.2 are P1+ *amplifiers*, dark until a downstream consumer drains the spool, and their v1 absence is an accepted degradation (static-only criticality, benchmark-only economics, hash-only staleness) — not a blocker.**

### 7.2 The feedback loop family

Each loop is a distinct mechanism the supply side consumes; enumerated so each gets built and monitored deliberately. **All five are P1+ amplifiers (ADR-0002): they light up when a downstream consumer drains the spool, and v1 depends on none of them.**

1. **Heat → criticality promotion.** Read frequency measures *exposure-if-wrong*: a bad fact in a hot concept damages many tasks. Heat therefore legitimately raises criticality scores, moving hot concepts (and their source files) toward the census stratum — including files not yet on any hand-maintained critical list. This is principled, not a heuristic.
2. **Miss → extraction demand.** `search_miss` is the concrete sensor for Supply §3's demand-driven processing. Repeated misses with a common query shape also reveal **ontology gaps** — a claim *type* the extractor doesn't produce yet — which feed Supply §16.3.
3. **Read-then-grep → distrust or insufficiency.** An agent consults a concept, then verifies against source anyway. For `claimed` tier: working as intended. For `verified`: either a content gap (concept didn't answer the question) or a trust failure — a gap detector nothing else provides.
4. **Outcome joins → live token economics.** Tasks with wiki hits vs. misses, matched by task class, turn Supply kill criterion 2 (tokens-per-task) from a one-time benchmark into a continuously updated measurement on real work.
5. **Contradiction → demotion.** The implementation of Supply §7.4: task outcomes contradicting a used concept demote it and queue re-extraction; confirmations promote. The mechanism finally has its sensor.

## 8. The Linter (`loam lint`)

Contradiction detection at the moment of introduction — the pipeline's machinery pointed at the time axis where resolution is cheapest. Catches humans and agents introducing changes that contradict existing knowledge (or existing knowledge that was already wrong).

### 8.1 Mechanical tier (instant)

Content-hash anchors make "which concepts cite the spans just changed" a lookup against the derived anchor index (Supply §4) with no LLM. `loam lint --staged` reports: *these concepts anchor to lines you changed; N are `verified`.* Catches doc-drift (function changed, doc concept now suspect) before commit instead of at the next nightly pass. Includes the deterministic structural checks from Supply §7.2 (e.g., signature changed while a concept asserts the old contract).

### 8.2 Semantic tier (on demand / on save)

**The T1.5 falsifier adapter running in reverse.** The falsifier asks "does this source support this claim?"; the linter asks the identical entailment question with fresh source: "does the *new* span still support the claims anchored to it?" Same model, same adapter, same task format — the multi-LoRA workhorse gets a second job for free. Runs over the anchored-claim set of dirty files only (small batches); never per-keystroke.

### 8.3 Contradiction is symmetric; the linter must not pick sides

New code contradicting an old doc may mean a contract is being broken — or that the doc was always wrong and the change is the correction. Output offers three exits: **fix the change**, **fix the other side**, or **`loam observe --concept-wrong <id>`**, filing the contradiction into the inbox with the edit as evidence. The third exit is load-bearing: the human at edit time has more context about which side is right than anyone will ever have again, and capturing that judgment as a provenance-tagged observation is far cheaper than the pipeline resolving the conflict cold later.

### 8.4 Ignoring the linter is not a no-op

The lint event is telemetry. If a warning is ignored and the change lands, the flagged concepts are demoted and queued for re-verification regardless, and the conflict becomes a first-class conflict object with the commit as evidence. The linter is not a gate — it is an early, optional offer to resolve a conflict while resolution is cheap; declined, the conflict flows into the normal machinery instead of festering. The store protects itself either way.

### 8.5 Advisory, or it gets uninstalled

Ships as an advisory pre-commit hook and a manual command. Never blocking in v1. `lint_outcome` telemetry records the disposition (fixed-change / fixed-other / filed-observation / ignored), which measures whether the linter earns its interruptions.

### 8.6 Future scope: CI/CD enforcement (out of scope, recorded)

Once lint precision is demonstrated by §8.5 telemetry, `loam lint` can run as a CI check — from advisory comment on PRs up to a required check for `verified`-tier contradictions. Explicitly **out of scope for v1**; noted here so the design (exit codes, `--json`, path filtering) keeps CI compatibility from day one.

### 8.7 Symmetry note

With the linter in place, every writer faces the same falsification discipline at its cheapest moment: the pipeline gates extracted knowledge, agent observations are gated by the pipeline, agent task-work is checked by lint-before-done, and human edits are checked at introduction. The knowledge store checks the work; the work checks the store.

## 9. Metrics and Kill Criteria

**Kill criteria for the consumption layer (numeric targets provisional until the P0 Experiment Protocol; the form is the commitment):**

1. **Consultation**: ≥70% of agent sessions on ingested corpora consult Loam before source exploration, per harness, after instruction iteration. Persistent failure means the integration model is wrong, not merely the stanzas.
2. **Usefulness**: bundle fallback rate — ≤30% of consumed bundles followed by immediate fallback exploration for the same question (loop §7.2.3). Persistent failure means extraction ontology or bundling is missing what agents actually need.
3. **Live economics**: outcome-joined token delta (loop §7.2.4) sustains the ≥30% reduction of Supply kill criterion 2 on real work, not just benchmarks. **This is the *continuous* form and is P1+ (ADR-0002); the one-time proof is the P0 paired-run benchmark, independent of any external telemetry consumer, whose target is sanity-checked against the measured rediscovery ceiling (ADR-0003).**
4. **Lint precision**: ≥60% of semantic-tier lint warnings result in an action (fix or filed observation) rather than ignore. Below that, the linter is noise and gets demoted to manual-only until fixed.

**Health metrics:** consultation rate per harness; miss rate and miss-query clustering; stale-read frequency; inbox volume by type and by harness; lint disposition distribution; read-then-grep rate by trust tier; heat-promotion queue size.

## 10. Phasing

- **P0 — Files + CLI core.** Bundle index conventions; `loam-core` library; `get` (with live anchor check), `search`, dumb-first `bundle` (§5.3 — it is the headline instruction of every stanza, so it ships with the stanzas, and the consultation baseline is measured against the real surface from day one), `observe` (inbox write); corpus resolution; telemetry emission to the local spool, flushing to the downstream consumer's event intake when available (§7.1 durability). Instruction stanzas for Claude Code and Hermes.
- **P1 — Loops.** Miss/heat/outcome loop consumers in the supply-side pipeline; consultation-rate dashboards; remaining harness stanzas. (These are the ADR-0002 amplifiers; they require a downstream consumer draining the P0 spool.)
- **P2 — Lint.** Mechanical tier; semantic tier via falsifier adapter; three-exit UX; advisory hook; disposition telemetry.
- **P3 — Data-gated extensions.** MCP wrapper for low-compliance harnesses (if any); ranking layer for `bundle` (if §9.2 demands it); procedural-queue handoff to skill distillation; CI/CD lint pilot.

## 11. Cross-References and Required Edits to Supply PRD

1. **Redaction gap — RESOLVED, with divergence (Supply §7.6).** This request originally asked for a redaction pass at write time. The adopted Redaction Contract goes further and differs deliberately: deterministic detection at S1 (before claims are cached — write-time-only redaction was judged insufficient because intermediate caches leak), declarative replacement tokens, a value-free registry, and versioned re-redaction sweeps. Preserved for the record; do not re-apply as written.
2. Supply §7.4 (use-time promotion/demotion) is implemented by §7.2.5 here.
3. Supply §3 (demand-driven processing) gets its sensor from §7.2.2 here.
4. Supply §8 strata gain a heat-derived criticality input from §7.2.1 here.
5. Supply §13.2 (context compilation) is `bundle`, governed by the dumb-first decision in §5.3.

## 12. Open Questions

1. Bundle size cap default and per-harness overrides (context budgets differ wildly across harnesses).
2. Inbox rate limiting / dedup per harness (a confused agent filing 400 identical observations should cost one re-verification, not 400).
3. `post_bundle_exploration` inference: derived from the downstream consumer's trace adjacency or explicitly signaled by well-behaved harnesses? (Probably both, with adjacency as the floor.)
4. Whether `lint` semantic tier requires the workhorse resident (GB10 up) or degrades gracefully to mechanical-only when the box is busy with a pipeline phase.
5. Stanza versioning: how instruction blocks in many repos stay current as the CLI evolves (likely: stanzas are generated artifacts, `loam init --refresh`).
