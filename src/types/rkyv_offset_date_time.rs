use rkyv::{Archive, Deserialize, Serialize};
use time::OffsetDateTime;

/// Archived layout of [`OffsetDateTime`]
#[derive(Archive, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[rkyv(remote = OffsetDateTime)]
pub struct RkyvOffsetDateTime {
    #[rkyv(getter = offset_date_time_unix_timestamp_nanos)]
    timestamp_nanos: i128,
}

impl From<OffsetDateTime> for RkyvOffsetDateTime {
    fn from(value: OffsetDateTime) -> Self {
        Self {
            timestamp_nanos: value.unix_timestamp_nanos(),
        }
    }
}

impl From<&OffsetDateTime> for RkyvOffsetDateTime {
    fn from(value: &OffsetDateTime) -> Self {
        Self {
            timestamp_nanos: value.unix_timestamp_nanos(),
        }
    }
}

impl From<RkyvOffsetDateTime> for OffsetDateTime {
    fn from(
        RkyvOffsetDateTime {
            timestamp_nanos,
        }: RkyvOffsetDateTime,
    ) -> Self {
        OffsetDateTime::from_unix_timestamp_nanos(timestamp_nanos).expect("always succeeds because timestamp_nanos originated from OffsetDateTime::unix_timestamp_nanos")
    }
}

fn offset_date_time_unix_timestamp_nanos(value: &OffsetDateTime) -> i128 {
    value.unix_timestamp_nanos()
}
