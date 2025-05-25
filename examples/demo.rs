use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    DefaultTerminal, Frame,
};

use rataudio_meter::{Meter, MeterInput, MeterState};

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
    let mut db_level = [-120.0; 4];

    let levels = generate_levels(100);
    let mut index_1 = 0;
    let mut index_2 = 10;
    let mut index_3 = 20;
    let mut index_4 = 50;

    let mut state = MeterState::default();

    loop {
        if last_time.elapsed() >= UPDATE_INTERVAL {
            db_level = [
                levels[index_1],
                levels[index_2],
                levels[index_3],
                levels[index_4],
            ];
            index_1 = (index_1 + 1) % levels.len();
            index_2 = (index_2 + 2) % levels.len();
            index_3 = (index_3 + 3) % levels.len();
            index_4 = (index_4 + 1) % levels.len();
            last_time = std::time::Instant::now();
        }

        terminal.draw(|frame| draw(frame, &db_level, &mut state))?;
        if handle_input()? == Command::Quit {
            break Ok(());
        }
    }
}

fn draw(frame: &mut Frame, db_level: &[f32], state: &mut MeterState) {
    let area = Rect::new(0, 1, frame.area().width, 3);
    let p = Paragraph::new("Rataudio Meter Demo")
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center);
    frame.render_widget(p, area);

    frame.render_widget(
        Block::bordered().border_type(BorderType::Rounded),
        Rect::new(9, 9, 63, 5),
    );

    frame.render_widget(
        Meter::mono()
            .show_labels(false)
            .show_scale(true)
            .db(MeterInput::Mono(db_level[0])),
        Rect::new(80, 10, 60, 3),
    );

    frame.render_widget(
        Meter::mono().db(MeterInput::Mono(db_level[0])),
        Rect::new(10, 10, 60, 3),
    );

    frame.render_widget(
        Meter::stereo().db(MeterInput::Stereo(db_level[1], db_level[2])),
        Rect::new(10, 16, 60, 5),
    );

    frame.render_stateful_widget(
        Meter::mono().db(MeterInput::Mono(db_level[3])),
        Rect::new(10, 24, 60, 3),
        state,
    );
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
