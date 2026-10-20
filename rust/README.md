# inductor-tui

A terminal calculator for hand-wound, single-layer air-core inductors, built with [Ratatui](https://ratatui.rs). It is the terminal counterpart of `../html/index.html` and implements the equations in `../docs/notes.md`.

Change any coil parameter and the inductance, self-capacitance, self-resonant frequency and a to-scale side view of the coil update immediately.

## Requirements

- Rust 1.88 or newer (the crate uses the 2024 edition and let-chains)
- A terminal at least ~90×26 with Unicode support (the side view uses braille characters) and 256 colours

## Running

```sh
cd rust
cargo run --release
```

To install it as `inductor-tui` on your `PATH`:

```sh
cargo install --path .
```

## Controls

| Key | Action |
| --- | --- |
| ↑ / ↓, `k` / `j`, Tab / Shift-Tab | Select a parameter |
| ← / →, `h` / `l` | Adjust the selected value by one step (cycles units and wire gauge) |
| Shift + ← / →, `H` / `L` | Adjust by ten steps |
| Enter, `e`, or start typing a digit | Type an exact value; Enter applies, Esc cancels |
| `u` | Cycle length units: mm → cm → in |
| `c` | Set coil length to close-wound (turns × wire diameter) |
| `o` | Set coil length equal to diameter (l/d = 1) |
| `t` | Set turns to the value needed for the target inductance |
| `q`, Esc, Ctrl-C | Quit |

Step sizes: 0.1 mm / 0.01 cm / 0.005 in for lengths, 0.25 for turns, 0.01 μH for the target, and one gauge for AWG. Choosing an AWG gauge sets the wire diameter; typing a diameter that matches no gauge shows "custom".

## What it shows

**Coil parameters**: units, coil diameter `d`, coil length `l`, turns `n`, wire diameter `d_wire`, wire gauge (AWG 10–36) and a target inductance.

**Results**

| Quantity | Equation |
| --- | --- |
| Inductance L (μH) | Wheeler: `L = d²n² / (18d + 40l)`, d and l in inches |
| Length / diameter | `l / d` |
| Medhurst H-factor | Interpolated from the table in the notes, clamped to l/d 1–5 |
| Self-capacitance Cs (pF) | `Cs = H · d`, d in cm |
| Self-resonant frequency | `f = 1 / (2π √(L·Cs))` |
| N_max at l/d = 1 | `d / d_wire` |
| Turn pitch, close-wound length, approximate wire length | `l / n`, `n · d_wire`, `n · π · (d + d_wire)` |

**Turns for target inductance**: `n = √(L·(18d + 40l)) / d` using the current diameter and length, plus whether those turns fit in the coil length.

**Warnings** appear when the turns don't fit in the coil length, when l/d is outside the H-factor table, or when l < 0.4·d (where Wheeler's formula loses accuracy).

**Side view**: a to-scale drawing of the coil form and windings with `l` and `d` dimensions. The winding turns red when the turns don't fit, and the drawing is capped at 400 turns.

## Development

```sh
cargo test     # physics and formatting unit tests
cargo clippy   # expected to be warning-free
```

| File | Contents |
| --- | --- |
| `src/calc.rs` | Physics: Wheeler, Medhurst H-factor, SRF, AWG diameters. All lengths in mm. |
| `src/app.rs` | Application state, editable fields, key handling, number formatting |
| `src/ui.rs` | Layout, tables, warnings, and the braille canvas side view |
| `src/main.rs` | Terminal setup and event loop |

Keep the equations consistent with `../docs/notes.md` and the HTML calculator. See `../CLAUDE.md` for the decisions behind them, such as why there are no Medhurst values below l/d = 1.
