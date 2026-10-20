# hand_rolled_inductors
For hand rolling inductors.

Tools for designing hand-wound, single-layer air-core inductors. Enter a coil's diameter, length, turns and wire size to get its inductance, self-capacitance and self-resonant frequency. You can also work backwards from a target inductance to the number of turns you need.

## Calculators

Both calculators use the same equations and show the same results.

| Calculator | Location | How to run |
| --- | --- | --- |
| Browser | [`html/index.html`](html/index.html) | Open the file in a browser. It is a single self-contained file with nothing to install. |
| Terminal (TUI) | [`rust/`](rust/) | `cd rust && cargo run --release` (needs Rust 1.88+). See the [TUI README](rust/README.md) for controls. |

Each calculator provides:

- Inductance from the Wheeler (ARRL) formula
- Self-capacitance from the Medhurst H-factor, and the self-resonant frequency
- Turns needed for a target inductance, and whether they fit in the coil length
- Unit switching (mm, cm, in) and an AWG wire-gauge picker
- Warnings for coils that won't fit or are outside the formulas' accurate range
- A to-scale side view of the coil

## Equations

[`docs/notes.md`](docs/notes.md) is the source of truth for the formulas: Wheeler inductance, the Medhurst H-factor table, N_max at l/d = 1, and the self-resonant frequency.
