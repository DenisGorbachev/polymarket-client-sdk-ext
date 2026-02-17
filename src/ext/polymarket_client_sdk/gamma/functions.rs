use crate::TIMESTAMP_2023_01_01_00_00_00_Z;
use chrono::{DateTime, Utc};

pub fn date_time_is_fresh(date_time: DateTime<Utc>) -> bool {
    date_time.timestamp() >= TIMESTAMP_2023_01_01_00_00_00_Z
}

pub fn option_date_time_is_fresh(date_time: Option<DateTime<Utc>>) -> bool {
    date_time.map(date_time_is_fresh).unwrap_or_default()
}
