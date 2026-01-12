use chrono::{DateTime, TimeZone, Utc};
use std::num::TryFromIntError;
use time::OffsetDateTime;

/// The offset of the returned `OffsetDateTime` is always `UTC`
pub fn from_chrono_date_time<Tz: TimeZone>(datetime: DateTime<Tz>) -> Result<OffsetDateTime, time::error::ComponentRange> {
    // Chrono: seconds since epoch + subsecond nanos
    let secs = datetime.timestamp() as i128;
    let subsec_nanos = datetime.timestamp_subsec_nanos() as i128;
    let unix_nanos = secs * 1_000_000_000 + subsec_nanos;
    OffsetDateTime::from_unix_timestamp_nanos(unix_nanos)
}

pub fn into_chrono_date_time(offset_date_time: OffsetDateTime) -> Result<DateTime<Utc>, TryFromIntError> {
    let unix_nanos = offset_date_time.unix_timestamp_nanos();
    i64::try_from(unix_nanos).map(DateTime::from_timestamp_nanos)
}
