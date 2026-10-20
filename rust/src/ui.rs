//! Rendering.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Block, Cell, Paragraph, Row, Table, Wrap,
        canvas::{self, Canvas, Rectangle},
    },
};

use crate::{
    app::{App, Field, fixed, fmt},
    calc::Results,
};

const ACCENT: Color = Color::Indexed(208);
const FORM: Color = Color::Indexed(137);
const BAD: Color = Color::Red;
const MUTED: Color = Color::DarkGray;

pub fn draw(frame: &mut Frame, app: &App) {
    let r = app.coil.calc();
    let warnings = warnings(app, &r);

    let lines: Vec<String> = warnings.iter().map(|w| format!("• {w}")).collect();
    // Height of the wrapped warnings plus borders; zero hides the strip.
    let wrap_width = frame.area().width.saturating_sub(2).max(1) as usize;
    let warn_rows: usize = lines
        .iter()
        .map(|l| l.chars().count().div_ceil(wrap_width))
        .sum();
    let warn_height = if lines.is_empty() {
        0
    } else {
        warn_rows as u16 + 2
    };

    let [title, body, warn, coil, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(14),
        Constraint::Length(warn_height),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    frame.render_widget(
        Line::from(vec![
            " Air-core inductor calculator ".bold().fg(ACCENT),
            "single-layer · Wheeler + Medhurst · docs/notes.md".fg(MUTED),
        ]),
        title,
    );

    let [left, results] =
        Layout::horizontal([Constraint::Length(46), Constraint::Min(40)]).areas(body);
    let [params, target] =
        Layout::vertical([Constraint::Length(9), Constraint::Min(5)]).areas(left);

    draw_params(frame, app, params);
    draw_target(frame, app, target);
    draw_results(frame, app, &r, results);
    if !lines.is_empty() {
        frame.render_widget(
            Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
                .wrap(Wrap { trim: true })
                .fg(Color::Yellow)
                .block(
                    Block::bordered()
                        .title(" Warnings ")
                        .border_style(Style::new().fg(Color::Yellow)),
                ),
            warn,
        );
    }
    draw_coil(frame, app, &r, coil);
    draw_footer(frame, app, footer);
}

fn draw_params(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Field::ALL.iter().enumerate().map(|(i, &f)| {
        let selected = i == app.selected;
        let editing = selected && app.editing.is_some();
        let value = match (&app.editing, selected) {
            (Some(buf), true) => format!("{buf}▏"),
            _ => field_value(app, f),
        };
        let unit = match f {
            Field::Unit | Field::Awg => String::new(),
            Field::Turns => "turns".into(),
            Field::Target => "μH".into(),
            _ => app.unit.label().into(),
        };
        let marker = if selected { "▶ " } else { "  " };
        let mut value_cell = Cell::from(Line::from(value).right_aligned());
        if editing {
            value_cell = value_cell.style(Style::new().black().on_yellow());
        } else if selected {
            value_cell = value_cell.style(Style::new().fg(ACCENT).bold());
        }
        let row = Row::new(vec![
            Cell::from(format!("{marker}{}", f.label())),
            Cell::from(f.symbol()).fg(MUTED),
            value_cell,
            Cell::from(unit).fg(MUTED),
        ]);
        if selected {
            row.add_modifier(Modifier::BOLD)
        } else {
            row
        }
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(5),
        ],
    )
    .block(Block::bordered().title(" Coil parameters "));
    frame.render_widget(table, area);
}

fn field_value(app: &App, f: Field) -> String {
    match f {
        Field::Unit => format!("◂ {} ▸", app.unit.label()),
        Field::Awg => match app.awg() {
            Some(g) => format!("◂ AWG {g} ▸"),
            None => "◂ custom ▸".into(),
        },
        _ => {
            let decimals = if f.is_length() {
                app.unit.decimals()
            } else {
                3
            };
            fixed(app.display_value(f), decimals)
        }
    }
}

fn result_table<'a>(rows: Vec<[String; 3]>, title: &'a str) -> Table<'a> {
    let rows = rows.into_iter().map(|[name, val, eq]| {
        Row::new(vec![
            Cell::from(name),
            Cell::from(Line::from(val).right_aligned())
                .fg(ACCENT)
                .bold(),
            Cell::from(eq).fg(MUTED),
        ])
    });
    Table::new(
        rows,
        [
            Constraint::Length(22),
            Constraint::Length(12),
            Constraint::Min(10),
        ],
    )
    .column_spacing(2)
    .block(Block::bordered().title(title))
}

fn draw_results(frame: &mut Frame, app: &App, r: &Results, area: Rect) {
    let rows = vec![
        [
            "Inductance L".into(),
            format!("{} μH", fmt(r.inductance_uh, 3)),
            "L = d²n² / (18d + 40l)  [in]".into(),
        ],
        ["Length / diameter".into(), fmt(r.ratio, 3), "l / d".into()],
        [
            "Medhurst H-factor".into(),
            fmt(r.h, 2),
            "table, clamped to 1–5".into(),
        ],
        [
            "Self-capacitance Cs".into(),
            format!("{} pF", fmt(r.cs_pf, 3)),
            "Cs = H · d  [cm]".into(),
        ],
        [
            "SRF".into(),
            format!("{} MHz", fmt(r.srf_hz / 1e6, 3)),
            "f = 1 / (2π √(L·Cs))".into(),
        ],
        [
            "N_max at l/d = 1".into(),
            fmt(r.n_max, 3),
            "N_max = d / d_wire".into(),
        ],
        ["Turn pitch".into(), app.len_fmt(r.pitch), "l / n".into()],
        [
            "Close-wound length".into(),
            app.len_fmt(r.close_len),
            "n · d_wire".into(),
        ],
        [
            "Wire needed (approx.)".into(),
            format!("{} cm", fmt(r.wire_len / 10.0, 3)),
            "n · π · (d + d_wire)".into(),
        ],
    ];
    frame.render_widget(result_table(rows, " Results "), area);
}

fn draw_target(frame: &mut Frame, app: &App, area: Rect) {
    let n = app.target_turns();
    let close = n * app.coil.w;
    let fits = close <= app.coil.l;
    let rows = vec![
        ["Turns required".into(), fmt(n, 3), String::new()],
        [
            "Close-wound length".into(),
            app.len_fmt(close),
            String::new(),
        ],
        [
            "Fits in length?".into(),
            if fits { "yes" } else { "no" }.into(),
            String::new(),
        ],
    ];
    let title = format!(" Turns for {} μH (t to apply) ", fmt(app.target_uh, 3));
    let table = Table::new(
        rows.into_iter().map(|[a, b, _]| {
            Row::new(vec![
                Cell::from(a),
                Cell::from(Line::from(b).right_aligned()).fg(ACCENT).bold(),
            ])
        }),
        [Constraint::Length(22), Constraint::Min(10)],
    )
    .column_spacing(2)
    .block(Block::bordered().title(title));
    frame.render_widget(table, area);
}

fn warnings(app: &App, r: &Results) -> Vec<String> {
    let mut out = Vec::new();
    if r.pitch < app.coil.w {
        out.push(format!(
            "Turns don't fit: {} turns of {} wire need at least {} of length.",
            fmt(app.coil.n, 3),
            app.len_fmt(app.coil.w),
            app.len_fmt(r.close_len)
        ));
    }
    if !(1.0..=5.0).contains(&r.ratio) {
        out.push(format!(
            "l/d = {} is outside the H-factor table (1–5); Cs uses the nearest end value.",
            fmt(r.ratio, 3)
        ));
    }
    if r.ratio < 0.4 {
        out.push("Wheeler's formula loses accuracy for l < 0.4·d (short, fat coils).".into());
    }
    out
}

/// Side view, to scale. Braille dots are roughly square, so one world unit
/// (mm) per dot keeps the proportions true.
fn draw_coil(frame: &mut Frame, app: &App, r: &Results, area: Rect) {
    let c = app.coil;
    let n_draw = c.n.ceil().min(400.0) as usize;
    let mut legend = format!(
        " {} turns, pitch {}, wire {} ",
        fmt(c.n, 3),
        app.len_fmt(r.pitch),
        app.len_fmt(c.w)
    );
    if (n_draw as f64) < c.n {
        legend.push_str(&format!("(drawing capped at {n_draw} turns) "));
    }
    let block = Block::bordered()
        .title(" Side view (to scale) ")
        .title_bottom(Line::from(legend).fg(MUTED));
    let inner = block.inner(area);

    let d_label = format!("d = {}", app.len_fmt(c.d));
    let l_label = format!("l = {}", app.len_fmt(c.l));
    // Dots available for the coil after leaving room for labels.
    let px = inner.width as f64 * 2.0;
    let py = inner.height as f64 * 4.0;
    let label_dots = (d_label.chars().count() as f64 + 6.0) * 2.0;
    let room_x = px - label_dots - 4.0;
    let room_y = py - 5.0 * 4.0; // one row above the coil, up to four below
    if room_x <= 8.0 || room_y <= 4.0 {
        frame.render_widget(
            Paragraph::new("terminal too small to draw")
                .fg(MUTED)
                .block(block),
            area,
        );
        return;
    }

    let wound = c.l.max(c.n * c.w);
    let form_len = wound + (0.15 * wound).max(2.0 * c.w);
    let s = (form_len / room_x).max(c.d / room_y); // mm per dot
    // Centre the coil plus its d label horizontally.
    let pad = ((px - form_len / s - label_dots) / 2.0).max(4.0);
    let x_min = -form_len / 2.0 - pad * s;
    // Centre vertically, keeping the rows reserved for the l dimension.
    let pad_y = ((py - c.d / s - 20.0) / 2.0).max(0.0);
    let y_max = c.d / 2.0 + (4.0 + pad_y) * s;
    let rows = inner.height as f64;
    let (top, bot) = (c.d / 2.0, -c.d / 2.0);
    let wire_color = if r.pitch < c.w { BAD } else { ACCENT };

    let canvas = Canvas::default()
        .block(block)
        .marker(Marker::Braille)
        .x_bounds([x_min, x_min + px * s])
        .y_bounds([y_max - py * s, y_max])
        .paint(move |ctx| {
            ctx.draw(&Rectangle::new(-form_len / 2.0, bot, form_len, c.d, FORM));
            ctx.layer();
            let x0 = -c.l / 2.0;
            for i in 0..n_draw {
                let xa = x0 + i as f64 * r.pitch + c.w / 2.0;
                let xb = xa + r.pitch * 0.5;
                ctx.draw(&canvas::Line::new(xa, top, xb, bot, wire_color));
            }
            ctx.layer();
            // Length dimension just below the coil, label in the row below that.
            let dim_y = bot - 6.0 * s;
            let tick = 1.5 * s;
            ctx.draw(&canvas::Line::new(x0, dim_y, x0 + c.l, dim_y, MUTED));
            ctx.draw(&canvas::Line::new(
                x0,
                dim_y - tick,
                x0,
                dim_y + tick,
                MUTED,
            ));
            ctx.draw(&canvas::Line::new(
                x0 + c.l,
                dim_y - tick,
                x0 + c.l,
                dim_y + tick,
                MUTED,
            ));
            let label_x = -(l_label.chars().count() as f64) * s; // centre: 2 dots per char
            // Shapes and labels round to cell rows differently, so place the
            // label by the row the dimension line actually landed in.
            let dim_row = ((y_max - dim_y) / (py * s) * (py - 1.0) / 4.0).floor();
            let label_row = (dim_row + 1.0).min(rows - 1.0);
            let label_y = y_max - (label_row + 0.5) * py * s / (rows - 1.0).max(1.0);
            ctx.print(
                label_x,
                label_y,
                Span::from(l_label.clone()).fg(Color::Gray),
            );
            // Diameter dimension right of the form.
            let dim_x = form_len / 2.0 + 4.0 * s;
            ctx.draw(&canvas::Line::new(dim_x, bot, dim_x, top, MUTED));
            ctx.draw(&canvas::Line::new(
                dim_x - tick,
                top,
                dim_x + tick,
                top,
                MUTED,
            ));
            ctx.draw(&canvas::Line::new(
                dim_x - tick,
                bot,
                dim_x + tick,
                bot,
                MUTED,
            ));
            ctx.print(
                dim_x + 4.0 * s,
                0.0,
                Span::from(d_label.clone()).fg(Color::Gray),
            );
        });
    frame.render_widget(canvas, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let line = if let Some(msg) = &app.message {
        Line::from(format!(" {msg}")).fg(Color::Yellow)
    } else if app.editing.is_some() {
        keys(&[
            ("0-9 . e", "type"),
            ("Enter", "apply"),
            ("Backspace", "delete"),
            ("Esc", "cancel"),
        ])
    } else {
        keys(&[
            ("↑↓", "select"),
            ("←→", "adjust"),
            ("⇧←→", "×10"),
            ("Enter/0-9", "type value"),
            ("u", "units"),
            ("c", "close-wound"),
            ("o", "l/d=1"),
            ("t", "use target turns"),
            ("q", "quit"),
        ])
    };
    frame.render_widget(line, area);
}

fn keys(pairs: &[(&'static str, &'static str)]) -> Line<'static> {
    let mut spans = vec![Span::raw(" ")];
    for (k, v) in pairs {
        spans.push(Span::from(*k).fg(ACCENT).bold());
        spans.push(Span::from(format!(" {v}  ")).fg(MUTED));
    }
    Line::from(spans)
}
