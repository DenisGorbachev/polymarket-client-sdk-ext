use chrono::{DateTime, TimeZone, Utc};
use time::OffsetDateTime;

/// The offset of the returned `OffsetDateTime` is always `UTC`
pub fn from_chrono_date_time<Tz: TimeZone>(datetime: DateTime<Tz>) -> Result<OffsetDateTime, time::error::ComponentRange> {
    // Chrono: seconds since epoch + subsecond nanos
    let secs = datetime.timestamp() as i128;
    let subsec_nanos = datetime.timestamp_subsec_nanos() as i128;
    let unix_nanos = secs * 1_000_000_000 + subsec_nanos;
    OffsetDateTime::from_unix_timestamp_nanos(unix_nanos)
}

pub fn into_chrono_date_time(offset_date_time: OffsetDateTime) -> DateTime<Utc> {
    let unix_nanos = offset_date_time.unix_timestamp_nanos();
    let unix_nanos = match i64::try_from(unix_nanos) {
        Ok(value) => value,
        Err(_) => {
            if unix_nanos.is_negative() {
                i64::MIN
            } else {
                i64::MAX
            }
        }
    };
    DateTime::from_timestamp_nanos(unix_nanos)
}
