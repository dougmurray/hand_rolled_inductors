//! Terminal calculator for hand-wound single-layer air-core inductors.

mod app;
mod calc;
mod ui;

use std::io;

use ratatui::crossterm::event::{self, Event, KeyEventKind};

use app::App;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| {
        let mut app = App::default();
        while !app.quit {
            terminal.draw(|frame| ui::draw(frame, &app))?;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                app.handle_key(key);
            }
        }
        Ok(())
    })
}
