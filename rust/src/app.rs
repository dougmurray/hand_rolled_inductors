//! Editable state and key handling.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::calc::{self, AWG_RANGE, Coil, MM_PER_IN};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    Mm,
    Cm,
    In,
}

impl Unit {
    pub fn mm(self) -> f64 {
        match self {
            Unit::Mm => 1.0,
            Unit::Cm => 10.0,
            Unit::In => MM_PER_IN,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Unit::Mm => "mm",
            Unit::Cm => "cm",
            Unit::In => "in",
        }
    }

    fn next(self) -> Self {
        match self {
            Unit::Mm => Unit::Cm,
            Unit::Cm => Unit::In,
            Unit::In => Unit::Mm,
        }
    }

    fn prev(self) -> Self {
        self.next().next()
    }

    /// Nudge step for lengths, in this unit.
    fn step(self) -> f64 {
        match self {
            Unit::Mm => 0.1,
            Unit::Cm => 0.01,
            Unit::In => 0.005,
        }
    }

    /// Decimals shown for lengths in this unit.
    pub fn decimals(self) -> usize {
        match self {
            Unit::In => 4,
            _ => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
    Unit,
    Diameter,
    Length,
    Turns,
    Wire,
    Awg,
    Target,
}

impl Field {
    pub const ALL: [Field; 7] = [
        Field::Unit,
        Field::Diameter,
        Field::Length,
        Field::Turns,
        Field::Wire,
        Field::Awg,
        Field::Target,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Field::Unit => "Length units",
            Field::Diameter => "Coil diameter",
            Field::Length => "Coil length",
            Field::Turns => "Turns",
            Field::Wire => "Wire diameter",
            Field::Awg => "Wire gauge",
            Field::Target => "Target inductance",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Field::Diameter => "d",
            Field::Length => "l",
            Field::Turns => "n",
            Field::Wire => "d_wire",
            Field::Target => "L",
            Field::Unit | Field::Awg => "",
        }
    }

    pub fn is_length(self) -> bool {
        matches!(self, Field::Diameter | Field::Length | Field::Wire)
    }

    /// Lower bound for nudging, in mm (lengths) or raw units.
    fn min(self) -> f64 {
        match self {
            Field::Wire => 0.05,
            Field::Target => 0.01,
            _ => 1.0,
        }
    }
}

pub struct App {
    pub coil: Coil,
    /// Target inductance, μH.
    pub target_uh: f64,
    pub unit: Unit,
    pub selected: usize,
    /// Text being typed into the selected field, if editing.
    pub editing: Option<String>,
    pub message: Option<String>,
    pub quit: bool,
}

impl Default for App {
    fn default() -> Self {
        App {
            coil: Coil {
                d: 20.0,
                l: 20.0,
                n: 20.0,
                w: 0.8,
            },
            target_uh: 1.0,
            unit: Unit::Mm,
            selected: 1,
            editing: None,
            message: None,
            quit: false,
        }
    }
}

impl App {
    pub fn field(&self) -> Field {
        Field::ALL[self.selected]
    }

    pub fn awg(&self) -> Option<u32> {
        calc::awg_for(self.coil.w)
    }

    pub fn target_turns(&self) -> f64 {
        self.coil.turns_for_uh(self.target_uh)
    }

    /// Raw stored value (mm for lengths).
    fn raw(&self, f: Field) -> f64 {
        match f {
            Field::Diameter => self.coil.d,
            Field::Length => self.coil.l,
            Field::Turns => self.coil.n,
            Field::Wire => self.coil.w,
            Field::Target => self.target_uh,
            Field::Awg => self.awg().map_or(f64::NAN, f64::from),
            Field::Unit => f64::NAN,
        }
    }

    fn set_raw(&mut self, f: Field, v: f64) {
        match f {
            Field::Diameter => self.coil.d = v,
            Field::Length => self.coil.l = v,
            Field::Turns => self.coil.n = v,
            Field::Wire => self.coil.w = v,
            Field::Target => self.target_uh = v,
            Field::Awg | Field::Unit => {}
        }
    }

    /// Value in display units.
    pub fn display_value(&self, f: Field) -> f64 {
        let v = self.raw(f);
        if f.is_length() { v / self.unit.mm() } else { v }
    }

    /// Formats a length (mm) in the current display unit.
    pub fn len_fmt(&self, mm: f64) -> String {
        format!("{} {}", fmt(mm / self.unit.mm(), 3), self.unit.label())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.message = None;
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if self.editing.is_some() {
            self.handle_edit_key(key);
            return;
        }
        let big = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                self.selected = (self.selected + Field::ALL.len() - 1) % Field::ALL.len();
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                self.selected = (self.selected + 1) % Field::ALL.len();
            }
            KeyCode::Left | KeyCode::Char('h') => self.nudge(-1.0, big),
            KeyCode::Right | KeyCode::Char('l') => self.nudge(1.0, big),
            KeyCode::Char('H') => self.nudge(-1.0, true),
            KeyCode::Char('L') => self.nudge(1.0, true),
            KeyCode::Enter | KeyCode::Char('e') => {
                if self.field() == Field::Unit {
                    self.unit = self.unit.next();
                } else {
                    self.editing = Some(String::new());
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                if self.field() != Field::Unit {
                    self.editing = Some(c.to_string());
                }
            }
            KeyCode::Char('u') => self.unit = self.unit.next(),
            KeyCode::Char('c') => self.coil.l = self.coil.n * self.coil.w,
            KeyCode::Char('o') => self.coil.l = self.coil.d,
            KeyCode::Char('t') => {
                let n = self.target_turns();
                if n.is_finite() {
                    self.coil.n = (n * 100.0).round() / 100.0;
                }
            }
            _ => {}
        }
    }

    fn handle_edit_key(&mut self, key: KeyEvent) {
        let Some(buf) = self.editing.as_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.editing = None,
            KeyCode::Backspace => {
                buf.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '-' | '+') => {
                buf.push(c);
            }
            KeyCode::Enter => {
                let text = self.editing.take().unwrap_or_default();
                self.commit(&text);
            }
            _ => {}
        }
    }

    fn commit(&mut self, text: &str) {
        let f = self.field();
        let Ok(v) = text.trim().parse::<f64>() else {
            self.message = Some(format!("“{text}” is not a number"));
            return;
        };
        if !(v.is_finite() && v > 0.0) {
            self.message = Some("Value must be greater than zero".into());
            return;
        }
        if f == Field::Awg {
            let g = v.round() as u32;
            if v.fract() != 0.0 || !AWG_RANGE.contains(&g) {
                self.message = Some(format!(
                    "Gauge must be a whole number from {} to {}",
                    AWG_RANGE.start(),
                    AWG_RANGE.end()
                ));
            } else {
                self.coil.w = calc::awg_mm(g);
            }
            return;
        }
        let raw = if f.is_length() { v * self.unit.mm() } else { v };
        self.set_raw(f, raw);
    }

    fn nudge(&mut self, dir: f64, big: bool) {
        let f = self.field();
        let mult = if big { 10.0 } else { 1.0 };
        match f {
            Field::Unit => {
                self.unit = if dir > 0.0 {
                    self.unit.next()
                } else {
                    self.unit.prev()
                }
            }
            Field::Awg => {
                // Higher gauge = thinner wire. From a custom diameter, snap to the nearest gauge.
                let g = self.awg().map_or_else(
                    || {
                        AWG_RANGE
                            .clone()
                            .min_by(|&a, &b| {
                                let da = (calc::awg_mm(a) - self.coil.w).abs();
                                let db = (calc::awg_mm(b) - self.coil.w).abs();
                                da.total_cmp(&db)
                            })
                            .unwrap_or(*AWG_RANGE.start())
                    },
                    |g| {
                        let next = g as i64 + (dir * mult) as i64;
                        next.clamp(*AWG_RANGE.start() as i64, *AWG_RANGE.end() as i64) as u32
                    },
                );
                self.coil.w = calc::awg_mm(g);
            }
            _ => {
                let step = match f {
                    Field::Turns => 0.25,
                    Field::Target => 0.01,
                    _ => self.unit.step(),
                };
                let v = self.display_value(f) + dir * step * mult;
                // Snap to the step grid so repeated nudges stay tidy.
                let v = (v / step).round() * step;
                let raw = if f.is_length() { v * self.unit.mm() } else { v };
                self.set_raw(f, raw.max(f.min()));
            }
        }
    }
}

/// Short human-friendly number, matching the HTML calculator's `fmt`.
pub fn fmt(x: f64, digits: usize) -> String {
    if !x.is_finite() {
        return "—".into();
    }
    if x == 0.0 {
        return "0".into();
    }
    let a = x.abs();
    if !(0.001..1000.0).contains(&a) {
        return format!("{x:.2e}");
    }
    let sig = digits + usize::from(a >= 100.0);
    let decimals = (sig as i32 - 1 - a.log10().floor() as i32).max(0) as usize;
    trim(format!("{x:.decimals$}"))
}

/// Fixed decimals with trailing zeros removed.
pub fn fixed(x: f64, decimals: usize) -> String {
    trim(format!("{x:.decimals$}"))
}

fn trim(s: String) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_matches_js() {
        assert_eq!(fmt(1.72413793, 3), "1.72");
        assert_eq!(fmt(123.456, 3), "123.5");
        assert_eq!(fmt(0.5, 3), "0.5");
        assert_eq!(fmt(20.0, 3), "20");
        assert_eq!(fmt(12345.0, 3), "1.23e4");
    }

    #[test]
    fn nudge_length_in_display_units() {
        let mut app = App {
            unit: Unit::Cm,
            selected: 1,
            ..App::default()
        };
        app.nudge(1.0, false);
        assert!((app.coil.d - 20.1).abs() < 1e-9);
    }

    #[test]
    fn awg_nudge_snaps_then_steps() {
        let mut app = App {
            selected: 5,
            ..App::default()
        };
        app.nudge(1.0, false);
        assert_eq!(app.awg(), Some(20));
        app.nudge(1.0, false);
        assert_eq!(app.awg(), Some(21));
    }
}
