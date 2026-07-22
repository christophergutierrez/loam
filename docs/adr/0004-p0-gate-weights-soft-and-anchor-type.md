# P0's gate rests on soft-claim precision and anchor/type discipline, not mechanical truth

**Status:** accepted · 2026-07-22

Mechanical claims have a deterministic production backstop (Supply §7.2): a wrong mechanical claim fails value-verification and is never admitted as `verified`, so extractor precision on mechanical *truth* governs **yield and wasted compute, not admitted-fact trust**. Soft claims have no backstop — cross-family falsification is their only gate. P0's gate is therefore re-weighted so its binding cost lands where the decision actually lives:

1. **Relax mechanical truth tolerance** — critical×mechanical 1% → 3% (matching standard×mechanical), or reframe the mechanical strata outright as a *yield/competence* readout ("≥X% of emitted mechanical claims survive verification") rather than a truth-precision gate.
2. **Hold a tight global anchor/type-correctness bound (≤1%) across all strata** — `bad_anchor` (claim points at the wrong span) and `bad_type` (a soft claim mistyped as mechanical dodges falsification forever) bypass every backstop, so 1% discipline belongs *here*, not on mechanical truth.
3. **Leave soft tolerances unchanged (3%/5%)** as the primary gate; the human-labeling budget (already soft-focused, §6) stays the binding P0 cost — correctly spent where the gate decision lives.

## Considered options

- **Keep mechanical at a 1% hard gate.** Rejected: it puts the tightest tolerance and biggest supply strain on the *lowest* admission-risk stratum (the one with a production backstop).
- **Tighten soft tolerances instead.** Rejected: explodes the human-labeling budget (1% → 300 clean human-labeled soft claims per candidate) for no supply relief.
- **Re-weight off mechanical, add a global anchor/type bound (chosen).**

## Consequences

- **Dissolves the critical×mechanical supply crunch** (expected 250–400 vs a 300-and-rising need): at 3% the need is ~100 clean, easily supplied.
- **Single-error sample inflation is explicitly a non-issue** (accepted): the real signal is *several* errors, handled by the **broken-stratum tripwire** (Glossary; BROKEN = Clopper-Pearson *lower* bound > tolerance ⇒ quarantine that stratum + alarm, healthy strata proceed), not by widening *n*.
- **Required-but-deferred edits:** P0 §5 (mechanical → yield readout or 3%; new global anchor/type ≤1% row; soft unchanged; rule-3 extension note becomes largely moot for mechanical); Supply §8 (add the three-state PASS/UNPROVEN/BROKEN verdict; §8.8 escalation is the UNPROVEN path).
- **Invariants preserved:** exact-binomial only (the tripwire just reads the same Clopper-Pearson interval from both ends); dual ledgers unaffected.
