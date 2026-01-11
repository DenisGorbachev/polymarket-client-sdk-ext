use chrono::{DateTime, TimeZone};
use time::OffsetDateTime;

/// The offset of the returned `OffsetDateTime` is always `UTC`
pub fn from_chrono_date_time<Tz: TimeZone>(dt: DateTime<Tz>) -> Result<OffsetDateTime, time::error::ComponentRange> {
    // Chrono: seconds since epoch + subsecond nanos
    let secs = dt.timestamp() as i128;
    let subsec_nanos = dt.timestamp_subsec_nanos() as i128;
    let unix_nanos = secs * 1_000_000_000 + subsec_nanos;
    OffsetDateTime::from_unix_timestamp_nanos(unix_nanos)
}
