use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{layout::Rect, DefaultTerminal, Frame};

use rataudio_meter::Meter;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn generate_levels(steps: usize) -> Vec<f32> {
    (0..steps)
        .map(|i| {
            let phase = i as f32 / steps as f32 * std::f32::consts::TAU;
            let normalized = phase.sin() * 0.5 + 0.5;
            -125.0 + normalized * 125.0
        })
        .collect()
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    const UPDATE_INTERVAL: Duration = Duration::from_millis(10);
    let mut last_time = std::time::Instant::now();
    let mut db_level = -120.0;

    let levels = generate_levels(100);
    let mut index = 0;

    loop {
        if last_time.elapsed() >= UPDATE_INTERVAL {
            db_level = levels[index];
            index = (index + 1) % levels.len();
            last_time = std::time::Instant::now();
        }

        terminal.draw(|frame| draw(frame, db_level))?;
        if handle_input()? == Command::Quit {
            break Ok(());
        }
    }
}

fn draw(frame: &mut Frame, db_level: f32) {
    frame.render_widget(Meter::default().db(db_level), Rect::new(10, 10, 60, 3));
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Noop,
    Quit,
}

fn handle_input() -> Result<Command> {
    if !event::poll(Duration::from_secs_f64(1.0 / 60.0))? {
        return Ok(Command::Noop);
    }
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => Ok(Command::Quit),
            _ => Ok(Command::Noop),
        },
        _ => Ok(Command::Noop),
    }
}
