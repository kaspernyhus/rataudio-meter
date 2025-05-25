//! The [`Meter`] widget is used to display a horizontal audio meter.
use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::{symbols, Buffer, Color, Rect, Widget},
    widgets::{Paragraph, StatefulWidget},
};

const MIN_DB: f32 = -120.0;
const YELLOW_START_DB: f32 = -12.0;
const RED_START_DB: f32 = -3.0;

/// A helper struct to convert between decibels and ratios for metering.
struct MeterScale {}

impl MeterScale {
    /// Determines the scale factor for the meter's logarithmic transformation.
    /// This factor is used to increase the resolution of the meter at higher dB values.
    const METER_LOG_SCALE_FACTOR: f32 = 2.0;

    /// Convert a decibel value to a ratio and apply an exponential transformation
    /// to increase meter resolution at higher db values
    pub fn db_to_ratio(db: f32) -> f32 {
        let db_ratio = 10_f32.powf(db / 20.0);
        let min_db_ratio = 10_f32.powf(MIN_DB / 20.0);
        let linear_ratio = (db_ratio.log10() - min_db_ratio.log10()) / (0.0 - min_db_ratio.log10());
        linear_ratio.powf(Self::METER_LOG_SCALE_FACTOR)
    }

    /// Convert a ratio to a decibel value and inverse the exponential transformation
    pub fn ratio_to_db(ratio: f32) -> f32 {
        let linear_ratio = ratio.powf(1.0 / Self::METER_LOG_SCALE_FACTOR);
        let min_db_ratio = 10_f32.powf(MIN_DB / 20.0);
        let db_ratio =
            10_f32.powf(linear_ratio * (0.0 - min_db_ratio.log10()) + min_db_ratio.log10());
        20.0 * db_ratio.log10()
    }

    #[allow(dead_code)]
    /// Convert a sample amplitude (between 0.0 and 1.0) to a decibel value.
    pub fn sample_to_db(sample_amplitude: f32) -> f32 {
        if sample_amplitude > 0.0 {
            20.0 * sample_amplitude.log10().clamp(MIN_DB, 0.0)
        } else {
            f32::NEG_INFINITY
        }
    }

    /// Convert a sample amplitude (between 0.0 and 1.0) to a ratio.
    pub fn sample_to_ratio(sample_amplitude: f32) -> f32 {
        if sample_amplitude <= 0.0 {
            return 0.0;
        }
        let l = MIN_DB / 20.0; // log10(min_db_ratio)
        let linear_ratio = (sample_amplitude.log10() - l) / -l;
        linear_ratio.powf(Self::METER_LOG_SCALE_FACTOR)
    }
}

lazy_static! {
    static ref YELLOW_START: f32 = MeterScale::db_to_ratio(self::YELLOW_START_DB);
    static ref RED_START: f32 = MeterScale::db_to_ratio(RED_START_DB);
    static ref LABEL_60: f32 = MeterScale::db_to_ratio(-60.0);
    static ref LABEL_40: f32 = MeterScale::db_to_ratio(-40.0);
    static ref LABEL_30: f32 = MeterScale::db_to_ratio(-30.0);
    static ref LABEL_24: f32 = MeterScale::db_to_ratio(-24.0);
    static ref LABEL_12: f32 = MeterScale::db_to_ratio(-12.0);
    static ref LABEL_6: f32 = MeterScale::db_to_ratio(-6.0);
    static ref LABEL_3: f32 = MeterScale::db_to_ratio(-3.0);
    static ref LABEL_0: f32 = MeterScale::db_to_ratio(-0.0);
}

/// A widget to display an audio meter.
///
/// A `Meter` renders a bar filled according to the value given to [`Meter::db`] or
/// [`Meter::ratio`]. The bar width and height are defined by the [`Rect`] it is
/// [rendered](Widget::render) in.
///
/// [`Meter`] is also a [`StatefulWidget`], which means you can use it with [`MeterState`] to allow
/// the meter to hold its peak value for a certain amount of time.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Meter {
    ratio: f32,
}

/// State of the [`Meter`] widget
///
/// This state can be used to render a peak hold. When the meter is rendered as a
/// stateful widget, it will mark the maximum peak for a certain amount of time. This will modify the [`MeterState`]
/// object passed to the`Frame::render_stateful_widget` method.
///
/// The state consists of:
/// - [`peak_hold_ratio`]: the peak value to be displayed
/// - [`peak_hold_time`]: the amount of time the peak value will be held
/// - [`last_peak_time`]: the time when the peak value was last updated
#[derive(Debug, Clone)]
pub struct MeterState {
    pub peak_hold_ratio: f32,
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
    /// Create a new [`Meter`] widget.
    pub fn new() -> Self {
        Self { ratio: 0.0 }
    }

    /// Set the value of the [`Meter`] widget in decibels relative to full scale.
    ///
    /// This method will saturate values above 0.0.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn db(mut self, dbfs: f32) -> Self {
        if (MIN_DB..=0.0).contains(&dbfs) {
            self.ratio = MeterScale::db_to_ratio(dbfs);
        } else {
            self.ratio = 0.0;
        }
        self
    }

    /// Set the value of the [`Meter`] widget from a sample amplitude value between 0.0 and 1.0.
    ///
    /// # Panics
    ///
    /// This method will panic if the value of `sample` is not between 0.0 and 1.0 inclusively.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn sample_amplitude(mut self, sample_amplitude: f32) -> Self {
        assert!(
            (0.0..=1.0).contains(&sample_amplitude),
            "Ratio should be between 0 and 1 inclusively."
        );
        self.ratio = MeterScale::sample_to_ratio(sample_amplitude);
        self
    }

    /// Set the value of the [`Meter`] widget as a ratio.
    ///
    /// `ratio` is the ratio between filled bar over empty bar (i.e. `3/4` completion is `0.75`).
    ///
    /// # Panics
    ///
    /// This method will panic if the value of `ratio` is not between 0.0 and 1.0 inclusively.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn ratio(mut self, ratio: f32) -> Self {
        assert!(
            (0.0..=1.0).contains(&ratio),
            "Ratio should be between 0 and 1 inclusively."
        );
        self.ratio = ratio;
        self
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

        // Split the area into three parts: db label, meter, and scale label
        let [db_area, meter_area, label_area] =
            Layout::vertical([Constraint::Max(1), Constraint::Min(1), Constraint::Max(1)])
                .areas(area);

        // Compute the start of the yellow and red zones
        let yellow_start = area.left() + (area.width as f32 * *YELLOW_START).round() as u16;
        let red_start = area.left() + (area.width as f32 * *RED_START).round() as u16;
        let end = area.left() + area.width;

        // Update the peak hold value
        let elapsed = state.last_peak_time.elapsed();
        if self.ratio > state.peak_hold_ratio {
            state.peak_hold_ratio = self.ratio;
            state.last_peak_time = Instant::now();
        } else if elapsed.as_secs() > state.peak_hold_time.as_secs() {
            state.peak_hold_ratio *= (0.99 - 0.01 * elapsed.as_secs_f32()).clamp(0.1, 0.99);
        }

        let peak_x =
            meter_area.left() + (f32::from(area.width) * state.peak_hold_ratio).round() as u16;

        // Render the meter
        for y in meter_area.top()..meter_area.bottom() {
            for x in meter_area.left()..end {
                if x <= meter_area.left() + (f32::from(area.width) * self.ratio).round() as u16 {
                    buf[(x, y)]
                        .set_symbol(symbols::block::SEVEN_EIGHTHS)
                        .set_fg(self.get_color(x, yellow_start, red_start));
                }
            }

            // Render the peak hold value
            buf[(peak_x, y)]
                .set_symbol(symbols::block::SEVEN_EIGHTHS)
                .set_fg(self.get_color(peak_x, yellow_start, red_start));
        }

        // Render the dB label
        let db_label = MeterScale::ratio_to_db(self.ratio);
        if db_label > MIN_DB {
            Paragraph::new(format!("{:.1} dB", db_label)).render(db_area, buf);
        } else {
            Paragraph::new("-∞ dB").render(db_area, buf);
        }

        // Render the scale labels
        self.render_meter_scale(label_area, buf);
    }

    fn render_meter_scale(&self, label_area: Rect, buf: &mut Buffer) {
        let total_width = label_area.width;

        if total_width > 50 {
            // Render all labels
            self.render_label_offset("-∞", 0.0, label_area, buf, 2, false);
            self.render_label("-60", *LABEL_60, label_area, buf);
            self.render_label("-40", *LABEL_40, label_area, buf);
            self.render_label("-24", *LABEL_24, label_area, buf);
            self.render_label("-12", *LABEL_12, label_area, buf);
            self.render_label("-6", *LABEL_6, label_area, buf);
            self.render_label("-3", *LABEL_3, label_area, buf);
            self.render_label("0", *LABEL_0, label_area, buf);
        } else if total_width > 30 {
            // Render fewer labels for medium-sized areas
            self.render_label_offset("-∞", 0.0, label_area, buf, 2, false);
            self.render_label("-60", *LABEL_60, label_area, buf);
            self.render_label("-40", *LABEL_40, label_area, buf);
            self.render_label("-24", *LABEL_24, label_area, buf);
            self.render_label("-12", *LABEL_12, label_area, buf);
            self.render_label("-3", *LABEL_3, label_area, buf);
            self.render_label("0", *LABEL_0, label_area, buf);
        } else {
            // Render minimal labels for small areas
            self.render_label_offset("-∞", 0.0, label_area, buf, 2, false);
            self.render_label("-60", *LABEL_60, label_area, buf);
            self.render_label("-30", *LABEL_30, label_area, buf);
            self.render_label("-12", *LABEL_12, label_area, buf);
            self.render_label("0", *LABEL_0, label_area, buf);
        }
    }

    // Helper functions to render individual labels
    fn render_label(&self, text: &str, ratio: f32, label_area: Rect, buf: &mut Buffer) {
        self.render_label_offset(text, ratio, label_area, buf, text.len() as u16, true);
    }

    fn render_label_offset(
        &self,
        text: &str,
        ratio: f32,
        label_area: Rect,
        buf: &mut Buffer,
        label_width: u16,
        offset: bool,
    ) {
        let x = if offset {
            label_area.left() + (label_area.width as f32 * ratio).round() as u16 - label_width / 2
        } else {
            label_area.left() + (label_area.width as f32 * ratio).round() as u16
        };

        Paragraph::new(text).render(
            Rect {
                x: x,
                y: label_area.top(),
                width: label_width,
                height: 1,
            },
            buf,
        );
    }

    fn get_color(&self, x: u16, yellow_start: u16, red_start: u16) -> Color {
        if x > red_start {
            Color::Red
        } else if x > yellow_start {
            Color::Yellow
        } else {
            Color::Green
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    #[test]
    fn test_db_to_ratio_at_zero() {
        let ratio = MeterScale::db_to_ratio(0.0);
        assert!((ratio - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_db_to_ratio_at_min_db() {
        let ratio = MeterScale::db_to_ratio(MIN_DB);
        assert!((ratio - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_ratio_to_db_inverts_db_to_ratio() {
        for db in [-120.0, -60.0, -20.0, -6.0, 0.0] {
            let ratio = MeterScale::db_to_ratio(db);
            let db_back = MeterScale::ratio_to_db(ratio);
            assert!(
                (db - db_back).abs() < 1.0,
                "db: {}, db_back: {}",
                db,
                db_back
            );
        }
    }

    #[test]
    fn test_sample_to_db_bounds() {
        assert_eq!(MeterScale::sample_to_db(0.0), f32::NEG_INFINITY);
        assert!((MeterScale::sample_to_db(1.0) - 0.0).abs() < EPSILON);
        assert!(MeterScale::sample_to_db(0.001) < -50.0);
    }

    #[test]
    fn test_sample_to_ratio_zero() {
        let ratio = MeterScale::sample_to_ratio(0.0);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_sample_to_ratio_full_scale() {
        let ratio = MeterScale::sample_to_ratio(1.0);
        assert!((ratio - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_sample_to_ratio_monotonicity() {
        let a = MeterScale::sample_to_ratio(0.01);
        let b = MeterScale::sample_to_ratio(0.1);
        let c = MeterScale::sample_to_ratio(1.0);
        assert!(a < b && b < c, "Ratios are not strictly increasing");
    }

    #[test]
    fn test_ratio_range_bounds() {
        for s in [0.001, 0.01, 0.1, 0.5, 1.0] {
            let ratio = MeterScale::sample_to_ratio(s);
            assert!(
                (0.0..=1.0).contains(&ratio),
                "Ratio out of bounds: {}",
                ratio
            );
        }
    }

    #[test]
    fn meter_invalid_db_upper_bound() {
        let meter = Meter::default().db(0.1);
        assert_eq!(meter.ratio, 0.0)
    }

    #[test]
    fn meter_db_zero() {
        let meter = Meter::default().db(0.0);
        assert_eq!(meter.ratio, 1.0);
    }

    #[test]
    fn meter_invalid_db_lower_bound() {
        let meter = Meter::default().db(-800.0);
        assert_eq!(meter.ratio, 0.0);
    }

    #[test]
    #[should_panic = "Ratio should be between 0 and 1 inclusively"]
    fn meter_invalid_ratio_upper_bound() {
        let _ = Meter::default().ratio(1.1);
    }

    #[test]
    #[should_panic = "Ratio should be between 0 and 1 inclusively"]
    fn meter_invalid_ratio_lower_bound() {
        let _ = Meter::default().ratio(-0.5);
    }
}
