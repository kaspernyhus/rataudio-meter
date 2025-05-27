use crate::scaling::MeterScale;
use lazy_static::lazy_static;

pub const MIN_DB: f32 = -120.0;
pub const YELLOW_START_DB: f32 = -12.0;
pub const RED_START_DB: f32 = -3.0;

lazy_static! {
    pub static ref YELLOW_START: f32 = MeterScale::db_to_ratio(self::YELLOW_START_DB);
    pub static ref RED_START: f32 = MeterScale::db_to_ratio(RED_START_DB);
    pub static ref LABEL_60: f32 = MeterScale::db_to_ratio(-60.0);
    pub static ref LABEL_40: f32 = MeterScale::db_to_ratio(-40.0);
    pub static ref LABEL_30: f32 = MeterScale::db_to_ratio(-30.0);
    pub static ref LABEL_24: f32 = MeterScale::db_to_ratio(-24.0);
    pub static ref LABEL_12: f32 = MeterScale::db_to_ratio(-12.0);
    pub static ref LABEL_6: f32 = MeterScale::db_to_ratio(-6.0);
    pub static ref LABEL_3: f32 = MeterScale::db_to_ratio(-3.0);
    pub static ref LABEL_0: f32 = MeterScale::db_to_ratio(-0.0);
}
