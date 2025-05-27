use std::time::{Duration, Instant};

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
    pub peak_hold_ratio: [f32; 2],
    pub last_peak_time: [Instant; 2],
    pub peak_hold_time: Duration,
}

impl Default for MeterState {
    fn default() -> Self {
        Self {
            peak_hold_ratio: [0.0; 2],
            last_peak_time: [Instant::now(); 2],
            peak_hold_time: Duration::from_secs(1),
        }
    }
}
