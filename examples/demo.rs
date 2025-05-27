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
    const UPDATE_INTERVAL: Duration = Duration::from_millis(20);
    let mut last_time = std::time::Instant::now();
    let mut db_level = [-120.0; 4];

    let levels = generate_levels(200);
    // let levels = [
    //     -800.0, -120.0, -60.0, -40.0, -30.0, -24.0, -12.0, -6.0, -3.0, 0.0, 0.1, 0.0, -3.0, -6.0,
    //     -12.0, -24.0, -30.0, -40.0, -60.0,
    // ];

    let mut index_1 = 0;
    let mut index_2 = 10;
    let mut index_3 = 20;
    let mut index_4 = 30;

    let mut states = [MeterState::default(), MeterState::default()];

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
            index_4 = (index_4 + 4) % levels.len();
            last_time = std::time::Instant::now();
        }

        terminal.draw(|frame| draw(frame, &db_level, &mut states))?;
        if handle_input()? == Command::Quit {
            break Ok(());
        }
    }
}

fn draw(frame: &mut Frame, db_level: &[f32], states: &mut [MeterState]) {
    let title_area = Rect::new(0, 1, frame.area().width, 3);
    let p = Paragraph::new("Rataudio Meter Demo")
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center);
    frame.render_widget(p, title_area);

    let m1 = Paragraph::new("Mono meter");
    frame.render_widget(m1, Rect::new(1, 6, 40, 1));

    frame.render_widget(
        Meter::mono().db(MeterInput::Mono(db_level[0])),
        Rect::new(1, 7, 80, 5),
    );

    let m2 = Paragraph::new("Mono meter - no labels, no scale");
    frame.render_widget(m2, Rect::new(1, 12, 40, 1));

    frame.render_widget(
        Meter::mono()
            .show_labels(false)
            .show_scale(false)
            .db(MeterInput::Mono(db_level[0])),
        Rect::new(1, 13, 22, 3),
    );

    frame.render_widget(
        Meter::stereo()
            .block(
                Block::default()
                    .title("Stereo Meter - no labels")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .show_labels(false)
            .db(MeterInput::Stereo(db_level[1], db_level[2])),
        Rect::new(0, 16, 60, 5),
    );

    let m3 = Paragraph::new("Meters with state - peak hold");
    frame.render_widget(m3, Rect::new(1, 24, 40, 1));

    frame.render_stateful_widget(
        Meter::mono().db(MeterInput::Mono(db_level[3])),
        Rect::new(1, 25, 40, 3),
        &mut states[0],
    );

    frame.render_stateful_widget(
        Meter::stereo().db(MeterInput::Stereo(db_level[1], db_level[2])),
        Rect::new(1, 30, 60, 5),
        &mut states[1],
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
