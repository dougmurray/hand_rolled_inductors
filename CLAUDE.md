# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Tools for designing hand-wound single-layer air-core inductors. `docs/notes.md` is the source of truth for the equations; two calculators implement them: `html/index.html` (browser) and `rust/` (Ratatui terminal UI). Browser calculator files belong in `html/`, the TUI in `rust/`. When an equation or warning changes, update both.

## Running and checking

The HTML calculator has no build, package manager, or test suite. `html/index.html` is a single self-contained file (inline CSS and JS, no external dependencies); open it directly in a browser.

To syntax-check the inline script:

```sh
sed -n '/<script>/,/<\/script>/p' html/index.html | sed '1d;$d' > /tmp/s.js && node --check /tmp/s.js
```

The TUI is a normal Cargo crate (run these from `rust/`):

```sh
cargo run            # launch the TUI
cargo test           # physics + formatting unit tests
cargo clippy         # should be warning-free
```

## Calculator architecture (HTML)

- `state` holds every input; all lengths are stored internally in **mm** and converted only for display (`UNIT_MM`, `toDisplay`/`fromDisplay`). Formulas convert to the units each equation expects: Wheeler uses **inches**, Medhurst Cs and N_max use **cm** (or a unitless ratio).
- `FIELDS` / `TARGET_FIELDS` describe the inputs; `buildRows` creates a synced slider + number box per field. Every change goes through `sync()` → `render()` → `drawCoil()`.
- `calc()` holds all the physics; `render()` builds the results tables and warnings from its output.

## TUI architecture (`rust/`)

- `calc.rs` holds the physics (`Coil::calc`, `h_factor`, `wheeler_uh`, `turns_for_uh`, `awg_mm`), the counterpart of the HTML `calc()`. `Coil` lengths are in **mm**, same unit rules as the HTML version.
- `app.rs` holds `App` state, the `Field` list, and key handling: nudge with ←/→ (Shift ×10), type values with Enter/digits. `fmt` matches the HTML `fmt` output.
- `ui.rs` renders the tables, the warnings strip, and a to-scale braille `Canvas` side view (one braille dot ≈ square, so lengths map to dots at a single scale).

## Equation decisions (keep these consistent with `docs/notes.md`)

- The turns-from-inductance formula in the notes is missing a parenthesis; the correct form is `n = sqrt(L·(18d + 40l)) / d`.
- `H_TABLE` mirrors the Medhurst H-factor table in the notes (l/d 1–5) and is clamped at its ends. Do **not** add Medhurst values for l/d < 1: the author rejected them because they push Cs error beyond ~9%.
- The approximate SRF formula `f ≈ 1/(n·√d)` from the notes is intentionally left out of the calculator: its units are unstated and it disagrees with the full `1/(2π√(LC))` result. Don't re-add it unless the author confirms its units.
