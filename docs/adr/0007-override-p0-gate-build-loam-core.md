# Build loam-core (Rust CLI) ahead of the P0 gate — deliberate override

**Status:** accepted · 2026-07-22 · **overrides a non-negotiable**

CLAUDE.md states the non-negotiable *"P0 (the model-floor experiment) gates all building."* The owner has **explicitly and knowingly overridden it** to begin building the demand-side `loam-core` CLI (Rust, ADR-0006) now, against a **fixture OKF bundle**, before P0 has run. This ADR records that the override was a conscious decision made with the gate in full view — not drift.

## Scope of the override

- **In:** `loam-core` library + the `loam` CLI commands `get` / `search` / `bundle` / `observe`, corpus resolution, read-time hash staleness, the telemetry spool — the Phase-2 consumption surface of `docs/IMPLEMENTATION_PLAN.md`, developed and tested against a hand-authored fixture bundle.
- **Out (still gated / deferred):** the supply pipeline (Phase 1), `loam lint`, the P0 experiment itself, and anything that consumes real P0 outputs (winning models, tolerances). The fixture bundle is a test scaffold, **not** a real extracted bundle — no extraction/verification is claimed.

## Why it's tolerable

The consumption surface is the one half of Loam whose *design* does not depend on the P0 outcome — the CLI reads an OKF bundle and emits telemetry regardless of which extractor produced the bundle. Building it early against a fixture de-risks the demand side and the artifact seam (ADR-0006) without asserting any verified knowledge. When P0 passes and Phase 1 produces a real bundle, the CLI already exists to consume it.

## Consequences

- CLAUDE.md's "currently docs-only" is no longer true; updated to note the override.
- Invariants still honored in the code: `agents-never-write` (the CLI never writes concepts — only the inbox), `derived-rebuildable` (spool/index are gitignored), `secrets-never-enter`, and the ADR-0006 artifact seam (Rust reads bundle/index/inbox/spool; no Python coupling).
- A future `VALIDATE`/audit should flag that code exists ahead of the P0 gate and point here for the rationale.
