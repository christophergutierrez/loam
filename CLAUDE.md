# Loam — Repo Instructions

Loam is a knowledge store + local extraction/verification pipeline for AI agents, running on **sunk-cost owned hardware reached through an endpoint abstraction** (reference config, not a requirement — e.g. a DGX Spark/GB10, H200s, or a large Mac; ADR-0005). The write path is the product: knowledge is admitted only through a verification gate with a measured error rate. Currently docs-only; P0 (the model-floor experiment) gates all building.

## Read first

- `docs/GLOSSARY.md` — canonical terms. If code or a doc conflicts with the glossary, the glossary wins; fix the other.
- `docs/SUPPLY_PRD.md` — extraction, verification, sampling, storage.
- `docs/CONSUMPTION_PRD.md` — CLI, telemetry, write boundary, linter.
- `docs/P0_EXPERIMENT_PROTOCOL.md` — the gate before component TRDs.
- `docs/adr/` — settled decisions that refine the PRDs (ADR-0001…0006). Where an ADR and older PRD prose disagree, the ADR wins (same rule as the glossary).

## Non-negotiables (settled; don't reopen without the owner)

1. Three namespaces, never mixed: model tiers **T0–T3**, pipeline stages **S0–S4**, roadmap phases **P0–P3**.
2. Same-base endorsement can never raise a claim above `claimed`; `corroborated` requires cross-family agreement by definition. The T1.5 falsifier is purely subtractive.
3. The T2 anchor model stays stock.
4. Exact binomial math only (Rule of Three, Clopper-Pearson, Beta-binomial). Wald/normal intervals are banned.
5. Random and directed sampling ledgers never mix denominators.
6. Consuming agents never write concepts — observations go to the inbox; the pipeline re-derives evidence.
7. Secrets never enter any Loam surface; redaction is declarative tokens, never masks; nothing derived from a secret value is stored.
8. Derived artifacts (anchor index, spool, caches) are rebuildable and never committed; the markdown bundle is canonical.

## Conventions

- Doc filenames carry no version numbers; version lives in the doc's frontmatter Status table.
- The name is "Loam" (a word, not an acronym) — never "LOAM".
- Cross-doc citations: "Supply §X" / "Consumption §X" / "Glossary".
- Kill-criteria numeric targets are provisional until the P0 protocol finalizes tolerances; the criteria's form is the commitment.
