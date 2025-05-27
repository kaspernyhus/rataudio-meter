//! The [`Meter`] widget is used to display a horizontal audio meter.

use crate::scaling::MeterScale;
use ratatui::widgets::Block;

/// Input type for the [`Meter`] widget
pub enum MeterInput {
    Mono(f32),
    Stereo(f32, f32),
}

/// A widget to display an audio meter.
///
/// A `Meter` renders a bar filled according to the value given to [`Meter::db`], [`Meter::sample_amplitude`] or
/// [`Meter::ratio`]. The bar width and height are defined by the [`Rect`] it is
/// [rendered](Widget::render) in.
///
/// [`Meter`] is also a [`StatefulWidget`], which means you can use it with [`MeterState`] to allow
/// the meter to hold its peak value for a certain amount of time.
#[derive(Debug, Clone, PartialEq)]
pub struct Meter<'a> {
    pub(crate) block: Option<Block<'a>>,
    pub(crate) ratio: [f32; 2],
    pub(crate) channels: usize,
    pub(crate) show_labels: bool,
    pub(crate) show_scale: bool,
}

impl<'a> Meter<'a> {
    /// Surrounds the `Meter` with a [`Block`].
    ///
    /// The meter is rendered in the inner portion of the block once space for borders and padding
    /// is reserved. Styles set on the block do **not** affect the meter itself.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Create a new mono [`Meter`] widget.
    pub fn mono() -> Self {
        Self {
            block: None,
            ratio: [0.0; 2],
            channels: 1,
            show_labels: true,
            show_scale: true,
        }
    }

    /// Create a new stereo [`Meter`] widget.
    pub fn stereo() -> Self {
        Self {
            block: None,
            ratio: [0.0; 2],
            channels: 2,
            show_labels: true,
            show_scale: true,
        }
    }

    /// Get the number of channels for this [`Meter`].
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Show or hide the decibel labels for the [`Meter`].
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Show or hide the scale below the [`Meter`].
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn show_scale(mut self, show: bool) -> Self {
        self.show_scale = show;
        self
    }

    /// Set the value of the [`Meter`] widget in decibels relative to full scale.
    /// This method will saturate values above 0.0dBFS to max.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn db(mut self, input: MeterInput) -> Self {
        match input {
            MeterInput::Mono(dbfs) => {
                self.ratio[0] = MeterScale::db_to_ratio(dbfs);
                self.ratio[1] = 0.0;
            }
            MeterInput::Stereo(left_dbfs, right_dbfs) => {
                self.ratio[0] = MeterScale::db_to_ratio(left_dbfs);
                self.ratio[1] = MeterScale::db_to_ratio(right_dbfs);
            }
        }
        self
    }

    /// Set the value of the [`Meter`] widget from a sample amplitude value between 0.0 and 1.0.
    /// This method will panic if the value of `sample` is not between 0.0 and 1.0 inclusively.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn sample_amplitude(mut self, input: MeterInput) -> Self {
        match input {
            MeterInput::Mono(ampl) => {
                assert!(
                    (0.0..=1.0).contains(&ampl),
                    "Ratio should be between 0 and 1 inclusively."
                );
                self.ratio[0] = MeterScale::sample_to_ratio(ampl);
                self.ratio[1] = 0.0;
            }
            MeterInput::Stereo(left_ampl, right_ampl) => {
                assert!(
                    (0.0..=1.0).contains(&left_ampl) && (0.0..=1.0).contains(&right_ampl),
                    "Ratio should be between 0 and 1 inclusively."
                );
                self.ratio[0] = MeterScale::sample_to_ratio(left_ampl);
                self.ratio[1] = MeterScale::sample_to_ratio(right_ampl);
            }
        }

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
    pub fn ratio(mut self, input: MeterInput) -> Self {
        match input {
            MeterInput::Mono(ratio) => {
                assert!(
                    (0.0..=1.0).contains(&ratio),
                    "Ratio should be between 0 and 1 inclusively."
                );
                self.ratio[0] = ratio;
                self.ratio[1] = 0.0;
            }
            MeterInput::Stereo(left_ratio, right_ratio) => {
                assert!(
                    (0.0..=1.0).contains(&left_ratio) && (0.0..=1.0).contains(&right_ratio),
                    "Ratio should be between 0 and 1 inclusively."
                );
                self.ratio[0] = left_ratio;
                self.ratio[1] = right_ratio;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_db_zero() {
        let meter = Meter::mono().db(MeterInput::Mono(0.0));
        assert_eq!(meter.ratio[0], 1.0)
    }

    #[test]
    fn meter_db_upper_bound() {
        let meter = Meter::mono().db(MeterInput::Mono(0.1));
        assert_eq!(meter.ratio[0], 1.0)
    }

    #[test]
    fn meter_db_lower_bound() {
        let meter = Meter::mono().db(MeterInput::Mono(-130.0));
        assert_eq!(meter.ratio[0], 0.0);
    }

    #[test]
    fn meter_stereo_db() {
        let meter = Meter::stereo().db(MeterInput::Stereo(0.0, 0.0));
        assert_eq!(meter.ratio[0], 1.0);
        assert_eq!(meter.ratio[1], 1.0);
    }

    #[test]
    fn meter_stereo_sample_amplitudes() {
        let meter = Meter::stereo().sample_amplitude(MeterInput::Stereo(1.0, 0.0));
        assert_eq!(meter.ratio[0], 1.0);
        assert_eq!(meter.ratio[1], 0.0);
    }

    #[test]
    #[should_panic = "Ratio should be between 0 and 1 inclusively"]
    fn meter_invalid_ratio_upper_bound() {
        let _ = Meter::mono().ratio(MeterInput::Mono(1.1));
    }

    #[test]
    #[should_panic = "Ratio should be between 0 and 1 inclusively"]
    fn meter_invalid_ratio_lower_bound() {
        let _ = Meter::mono().ratio(MeterInput::Mono(-0.5));
    }
}
