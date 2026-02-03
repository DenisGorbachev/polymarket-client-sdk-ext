use rkyv::{Archive, Deserialize, Serialize};
use time::Duration;

/// Archived layout of [`Duration`]
#[derive(Archive, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[rkyv(remote = Duration)]
pub struct RkyvDuration {
    #[rkyv(getter = duration_whole_seconds)]
    seconds: i64,
    #[rkyv(getter = duration_subsec_nanoseconds)]
    nanoseconds: i32,
}

impl From<Duration> for RkyvDuration {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.whole_seconds(),
            nanoseconds: value.subsec_nanoseconds(),
        }
    }
}

impl From<&Duration> for RkyvDuration {
    fn from(value: &Duration) -> Self {
        Self {
            seconds: value.whole_seconds(),
            nanoseconds: value.subsec_nanoseconds(),
        }
    }
}

impl From<RkyvDuration> for Duration {
    fn from(
        RkyvDuration {
            seconds,
            nanoseconds,
        }: RkyvDuration,
    ) -> Self {
        Duration::new(seconds, nanoseconds)
    }
}

fn duration_whole_seconds(value: &Duration) -> i64 {
    value.whole_seconds()
}

fn duration_subsec_nanoseconds(value: &Duration) -> i32 {
    value.subsec_nanoseconds()
}
