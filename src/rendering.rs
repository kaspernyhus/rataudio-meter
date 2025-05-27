use std::{cmp::min, time::Instant};

use ratatui::{
    layout::{Constraint, Layout},
    prelude::{symbols, BlockExt, Buffer, Color, Rect, Widget},
    widgets::{Paragraph, StatefulWidget},
};

use crate::meter::Meter;
use crate::state::MeterState;
use crate::{
    constants::{
        LABEL_0, LABEL_12, LABEL_24, LABEL_3, LABEL_30, LABEL_40, LABEL_6, LABEL_60, MIN_DB,
        RED_START, YELLOW_START,
    },
    scaling::MeterScale,
};

impl Widget for Meter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &Meter<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = MeterState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

impl StatefulWidget for Meter<'_> {
    type State = MeterState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(&self, area, buf, state);
    }
}

impl StatefulWidget for &Meter<'_> {
    type State = MeterState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(block) = self.block.as_ref() {
            block.render(area, buf);
        }

        let meter_area = self.block.inner_if_some(area);
        if meter_area.is_empty() {
            return;
        }

        // Prepare areas for meter(s), labels and scale if enabled
        let mut layout_constraints = Vec::new();
        if self.show_labels {
            for _ in 0..self.channels {
                layout_constraints.push(Constraint::Length(1));
            }
        }
        for _ in 0..self.channels {
            layout_constraints.push(Constraint::Length(1));
        }
        if self.show_scale {
            layout_constraints.push(Constraint::Length(1));
        }
        let layout_areas = Layout::vertical(layout_constraints).split(meter_area);

        let mut index = 0;
        let db_areas = if self.show_labels {
            let a = &layout_areas[index..index + self.channels];
            index += self.channels;
            Some(a)
        } else {
            None
        };

        let meter_areas = &layout_areas[index..index + self.channels];
        index += self.channels;

        let scale_area = if self.show_scale {
            Some(layout_areas[index])
        } else {
            None
        };

        let meter_width = meter_area.width as f32;

        // Compute color zones (same for all channels)
        // There should be at least 1 bar yellow and 1 bar red for the rightmost meter bars.
        let end = meter_areas[0].left() + meter_width as u16;
        let yellow_start = min(
            meter_areas[0].left() + (meter_width * *YELLOW_START).round() as u16,
            end - 2,
        );
        let red_start = min(
            meter_areas[0].left() + (meter_width * *RED_START).round() as u16,
            end - 1,
        );

        for channel in 0..self.channels {
            let ratio = self.ratio[channel];

            // --- METER BARS ---
            let y = meter_areas[channel].y;
            for x in meter_areas[channel].left()..end {
                if x <= meter_areas[channel].left() + (meter_width * ratio).round() as u16 {
                    buf[(x, y)]
                        .set_symbol(symbols::block::SEVEN_EIGHTHS)
                        .set_fg(self.get_color(x, yellow_start, red_start));
                }
            }

            // --- PEAK HOLD ---
            let elapsed = state.last_peak_time[channel].elapsed();
            if ratio > state.peak_hold_ratio[channel] {
                state.peak_hold_ratio[channel] = ratio;
                state.last_peak_time[channel] = Instant::now();
            } else if elapsed.as_secs_f32() > state.peak_hold_time.as_secs_f32() {
                state.peak_hold_ratio[channel] *=
                    (0.99 - 0.01 * elapsed.as_secs_f32()).clamp(0.1, 0.99);
            }

            // --- PEAK MARKER ---
            let raw_peak_x = meter_areas[channel].left()
                + (meter_width * state.peak_hold_ratio[channel]).round() as u16;
            let peak_x = raw_peak_x.clamp(meter_areas[channel].left(), end - 1);

            buf[(peak_x, y)]
                .set_symbol(symbols::block::SEVEN_EIGHTHS)
                .set_fg(self.get_color(peak_x, yellow_start, red_start));

            // --- DB LABEL ---
            if let Some(db_areas) = db_areas {
                let db_area = db_areas[channel];
                let db_label = MeterScale::ratio_to_db(ratio);
                let text = if db_label > MIN_DB {
                    format!("{:.1} dB", db_label)
                } else {
                    "-∞ dB".to_string()
                };
                Paragraph::new(text).render(db_area, buf);
            }
        }

        // --- SCALE LABELS ---
        if let Some(scale_area) = scale_area {
            self.render_meter_scale(scale_area, buf);
        }
    }
}

impl Meter<'_> {
    fn render_meter_scale(&self, label_area: Rect, buf: &mut Buffer) {
        let total_width = label_area.width;
        if total_width > 50 {
            // Render all labels
            self.render_scale_label("-∞", 0.0, label_area, buf, Some(1));
            self.render_scale_label("-60", *LABEL_60, label_area, buf, None);
            self.render_scale_label("-40", *LABEL_40, label_area, buf, None);
            self.render_scale_label("-24", *LABEL_24, label_area, buf, None);
            self.render_scale_label("-12", *LABEL_12, label_area, buf, None);
            self.render_scale_label("-6", *LABEL_6, label_area, buf, None);
            self.render_scale_label("-3", *LABEL_3, label_area, buf, None);
            self.render_scale_label("0", *LABEL_0, label_area, buf, None);
        } else if total_width > 35 {
            // Render fewer labels for medium-sized areas
            self.render_scale_label("-∞", 0.0, label_area, buf, Some(1));
            self.render_scale_label("-60", *LABEL_60, label_area, buf, None);
            self.render_scale_label("-40", *LABEL_40, label_area, buf, None);
            self.render_scale_label("-24", *LABEL_24, label_area, buf, None);
            self.render_scale_label("-12", *LABEL_12, label_area, buf, None);
            self.render_scale_label("-6", *LABEL_6, label_area, buf, Some(1));
            self.render_scale_label("0", *LABEL_0, label_area, buf, None);
        } else if total_width > 20 {
            // Render minimal labels for small areas
            self.render_scale_label("-∞", 0.0, label_area, buf, Some(1));
            self.render_scale_label("-60", *LABEL_60, label_area, buf, None);
            self.render_scale_label("-30", *LABEL_30, label_area, buf, None);
            self.render_scale_label("-12", *LABEL_12, label_area, buf, None);
            self.render_scale_label("0", *LABEL_0, label_area, buf, None);
        } else {
            // Render least labels for small areas
            self.render_scale_label("-∞", 0.0, label_area, buf, Some(1));
            self.render_scale_label("-60", *LABEL_60, label_area, buf, None);
            self.render_scale_label("-30", *LABEL_30, label_area, buf, None);
            self.render_scale_label("0", *LABEL_0, label_area, buf, None);
        }
    }

    fn render_scale_label(
        &self,
        text: &str,
        ratio: f32,
        label_area: Rect,
        buf: &mut Buffer,
        offset: Option<i16>,
    ) {
        let offset = offset.unwrap_or(0);
        let label_base = label_area.left() as i16 - 1 + offset;
        let label_start = (label_area.width as f32 * ratio).round() as i16;
        let x = (label_base + label_start) as u16;

        Paragraph::new(text).render(
            Rect {
                x,
                y: label_area.y,
                width: label_area.width,
                height: 1,
            },
            buf,
        );
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
}
