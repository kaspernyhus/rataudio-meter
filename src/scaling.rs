use crate::constants::MIN_DB;

pub struct MeterScale {}

/// A helper struct to convert between decibels and ratios for metering.
impl MeterScale {
    /// Determines the scale factor for the meter's logarithmic transformation.
    /// This factor is used to increase the resolution of the meter at higher dB values.
    const METER_LOG_SCALE_FACTOR: f32 = 2.0;

    /// Convert a decibel value to a ratio
    pub fn db_to_ratio(db: f32) -> f32 {
        if db <= MIN_DB {
            return 0.0;
        }
        if db >= 0.0 {
            return 1.0;
        }

        let db_ratio = 10_f32.powf(db / 20.0);
        let min_db_ratio = 10_f32.powf(MIN_DB / 20.0);
        let linear_ratio = (db_ratio.log10() - min_db_ratio.log10()) / (0.0 - min_db_ratio.log10());
        linear_ratio.powf(Self::METER_LOG_SCALE_FACTOR)
    }

    /// Convert a ratio to a decibel value
    pub fn ratio_to_db(ratio: f32) -> f32 {
        let linear_ratio = ratio.powf(1.0 / Self::METER_LOG_SCALE_FACTOR);
        let min_db_ratio = 10_f32.powf(MIN_DB / 20.0);
        let db_ratio =
            10_f32.powf(linear_ratio * (0.0 - min_db_ratio.log10()) + min_db_ratio.log10());
        20.0 * db_ratio.log10()
    }

    /// Convert a sample amplitude (between 0.0 and 1.0) to a decibel value.
    #[allow(dead_code)]
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
        if sample_amplitude >= 1.0 {
            return 1.0;
        }

        let l = MIN_DB / 20.0; // log10(min_db_ratio)
        let linear_ratio = (sample_amplitude.log10() - l) / -l;
        linear_ratio.powf(Self::METER_LOG_SCALE_FACTOR)
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
    fn test_db_to_ratio_below_min_db() {
        let ratio = MeterScale::db_to_ratio(MIN_DB - 100.0);
        println!("ratio: {}", ratio);
        assert!((ratio - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_db_to_ratio_above_max_db() {
        let ratio = MeterScale::db_to_ratio(0.1);
        println!("ratio: {}", ratio);
        assert!((ratio - 1.0).abs() < EPSILON);
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
    fn test_sample_to_ratio_invalid_above() {
        let ratio = MeterScale::sample_to_ratio(1.1);
        assert!((ratio - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_sample_to_ratio_invalid_below() {
        let ratio = MeterScale::sample_to_ratio(-0.1);
        assert!((ratio - 0.0).abs() < EPSILON);
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
}
