# loam-core is Rust (demand side); the pipeline is Python (supply side), distinct across an artifact seam

**Status:** accepted · 2026-07-22

The consumption surface — `loam-core` and the `loam` CLI (`get`/`search`/`bundle`/`observe`/`lint`) — is written in **Rust**. The supply side — extraction, verification, model serving, sampling, scoring, adapter training, and the P0 experiments — is written in **Python**. The "never Rust for services" rule that briefly shadowed this decision came *only* from the sdlc plugin's VA-conventions table, which has been removed from this project; it does not apply to Loam.

The two sides are **distinct and never share a process or in-memory API.** They communicate only through durable artifacts, which is *why* the language split is clean (no FFI, no shared runtime, no RPC coupling):

- the git-resident **OKF markdown bundle** — canonical, the contract between supply and demand;
- the derived **SQLite anchor index** — rebuildable, read by `loam-core`, rebuilt by the pipeline (never a source of truth, per non-negotiable 8);
- the **inbox** queue — `loam observe` writes it; the pipeline drains it;
- the **telemetry spool** — `loam-core` writes it; TraceStore (P1+) drains it.

`loam-core` *reads* the bundle + index and *writes* the inbox + spool; the pipeline *writes* the bundle + rebuilds the index and *reads* the inbox. Neither calls the other's code.

## Rationale

- **Rust for the demand side:** agents invoke the CLI constantly (before exploration, per concept, before done), so cold-start latency dominates — a static Rust binary starts in ~ms where a Python CLI pays interpreter/import warmup every call. It ships as one dependency-free binary into any harness's PATH (the heterogeneous, "not ours" consumer set), and its hot paths are CPU-bound (microsecond content-hash verification at read time, SQLite index lookups, the append-only spool).
- **Python for the supply side:** the ML ecosystem (vLLM serving, QLoRA/PEFT, tokenizers, exact-binomial scoring libs) lives there.

## Considered options

- **Single language (all-Python or all-Rust).** Rejected: Python is weak for the ubiquitous low-latency CLI; Rust is weak for the ML pipeline. The workloads have opposite centers of gravity.
- **Shared process / FFI (PyO3 or a Rust extension in Python).** Rejected: it couples the two sides, defeats the ship-everywhere CLI, and destroys the clean supply↔demand seam.

## Consequences

- **Design-time distinctness (the constraint):** the supply↔demand boundary must stay **artifact-only** — bundle, index, inbox, spool. Any proposal to share code, link a library, or add a synchronous Rust↔Python call across that boundary is a violation to flag. Keeping them distinct is a stated design goal, not an accident.
- **Implementation-time seam testing:** the artifact boundary *is* the highest practical test seam, which is exactly what the Killhouse pipeline already tests at (`to-prd` picks the highest seam; `PLAN` mandates a tracer bullet through it; `IMPLEMENT_MILESTONE` writes the test at that seam). No extra machinery needed.
- `Cargo.lock` is committed (binary crate); Python deps pinned separately. The `.gitignore` already anticipates both stacks.
- A future Killhouse `VALIDATE` loop (see the killhouse backlog) could later check that no cross-seam coupling crept in.
- **Invariants preserved:** the markdown bundle stays canonical; derived artifacts (index, spool, caches) remain rebuildable and uncommitted (non-negotiable 8).
