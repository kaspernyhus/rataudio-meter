use std::time::{Duration, Instant};

use ratatui::{
    layout::{Constraint, Layout, Offset},
    prelude::{symbols, Buffer, Color, Rect, Widget},
    widgets::{Paragraph, StatefulWidget},
};

const MIN_DB: f64 = -120.0;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Meter {
    ratio: f64,
}

#[derive(Debug, Clone)]
pub struct MeterState {
    pub peak_hold_ratio: f64,
    pub peak_hold_time: Duration,
    pub last_peak_time: Instant,
}

impl Default for MeterState {
    fn default() -> Self {
        Self {
            peak_hold_ratio: 0.0,
            peak_hold_time: Duration::from_secs(1),
            last_peak_time: Instant::now(),
        }
    }
}

impl Meter {
    pub fn new() -> Self {
        Self { ratio: 0.0 }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn db(mut self, db: f64) -> Self {
        assert!(
            (-120.0..0.0).contains(&db),
            "dB value should be between -120.0 and 0.0 inclusively."
        );
        self.ratio = Self::ratio_from_db(&db);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn ratio(mut self, ratio: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&ratio),
            "Ratio should be between 0 and 1 inclusively."
        );
        self.ratio = ratio;
        self
    }

    pub fn ratio_from_db(db: &f64) -> f64 {
        let db_ratio = 10_f64.powf(db / 20.0);
        let min_db_ratio = 10_f64.powf(MIN_DB / 20.0);
        let linear_ratio = (db_ratio.log10() - min_db_ratio.log10()) / (0.0 - min_db_ratio.log10());
        linear_ratio.powf(2.0)
    }

    pub fn db_from_ratio(ratio: &f64) -> f64 {
        let linear_ratio = ratio.powf(1.0 / 2.0);
        let min_db_ratio = 10_f64.powf(MIN_DB / 20.0);
        let db_ratio =
            10_f64.powf(linear_ratio * (0.0 - min_db_ratio.log10()) + min_db_ratio.log10());
        20.0 * db_ratio.log10()
    }
}

impl Widget for Meter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_meter(area, buf, &mut MeterState::default());
    }
}

impl StatefulWidget for Meter {
    type State = MeterState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_meter(area, buf, state);
    }
}

impl Meter {
    fn render_meter(&self, area: Rect, buf: &mut Buffer, state: &mut MeterState) {
        if area.is_empty() {
            return;
        }

        let [db_area, meter_area, label_area] =
            Layout::vertical([Constraint::Max(1), Constraint::Min(1), Constraint::Max(1)])
                .areas(area);

        let total_width = area.width;
        let yellow_start =
            area.left() + (total_width as f64 * Self::ratio_from_db(&-12.0f64)).round() as u16;
        let red_start =
            area.left() + (total_width as f64 * Self::ratio_from_db(&-3.0f64)).round() as u16;
        let end = area.left() + area.width;

        let elapsed = state.last_peak_time.elapsed();
        if self.ratio > state.peak_hold_ratio {
            state.peak_hold_ratio = self.ratio;
            state.last_peak_time = Instant::now();
        } else if elapsed.as_secs() > state.peak_hold_time.as_secs() {
            state.peak_hold_ratio *= (0.99 - 0.01 * elapsed.as_secs_f64()).clamp(0.1, 0.99);
        }

        let peak_x =
            meter_area.left() + (f64::from(total_width) * state.peak_hold_ratio).round() as u16;

        for y in meter_area.top()..meter_area.bottom() {
            for x in meter_area.left()..end {
                if x <= meter_area.left() + (f64::from(total_width) * self.ratio).round() as u16 {
                    buf[(x, y)]
                        .set_symbol(symbols::block::SEVEN_EIGHTHS)
                        .set_fg(self.get_color(x, yellow_start, red_start));
                }
            }

            buf[(peak_x, y)]
                .set_symbol(symbols::block::SEVEN_EIGHTHS)
                .set_fg(self.get_color(peak_x, yellow_start, red_start));
        }

        Paragraph::new(format!("{:.1} dB", Self::db_from_ratio(&self.ratio),)).render(db_area, buf);
        Paragraph::new("-inf").render(
            self.get_label_position(label_area, total_width, MIN_DB),
            buf,
        );
        Paragraph::new("-60").render(
            self.get_label_position(label_area, total_width, -60.0)
                .offset(Offset { x: -1, y: 0 }),
            buf,
        );
        Paragraph::new("-40").render(
            self.get_label_position(label_area, total_width, -40.0)
                .offset(Offset { x: -1, y: 0 }),
            buf,
        );
        Paragraph::new("-24").render(
            self.get_label_position(label_area, total_width, -24.0)
                .offset(Offset { x: -1, y: 0 }),
            buf,
        );
        Paragraph::new("-12").render(
            self.get_label_position(label_area, total_width, -12.0)
                .offset(Offset { x: -1, y: 0 }),
            buf,
        );
        Paragraph::new("-6").render(
            self.get_label_position(label_area, total_width, -6.0)
                .offset(Offset { x: -1, y: 0 }),
            buf,
        );
        Paragraph::new("-3").render(
            self.get_label_position(label_area, total_width, -3.0)
                .offset(Offset { x: -1, y: 0 }),
            buf,
        );
        Paragraph::new("0").render(self.get_label_position(label_area, total_width, 0.0), buf);
    }

    fn get_color(&self, x: u16, yellow_start: u16, red_start: u16) -> Color {
        if x >= red_start {
            Color::Red
        } else if x >= yellow_start {
            Color::Yellow
        } else {
            Color::Green
        }
    }

    fn get_label_position(&self, label_area: Rect, total_width: u16, db: f64) -> Rect {
        Rect {
            x: label_area.left() + (total_width as f64 * Self::ratio_from_db(&db)).round() as u16,
            y: label_area.top(),
            width: 4,
            height: 1,
        }
    }
}

// Todo: Add tests for the meter widget
