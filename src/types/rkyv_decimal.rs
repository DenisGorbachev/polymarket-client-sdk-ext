use rkyv::{Archive, Deserialize, Serialize};
use rust_decimal::Decimal;

/// Archived layout of [`Decimal`]
#[derive(Archive, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[rkyv(remote = Decimal)]
pub struct RkyvDecimal {
    #[rkyv(getter = Decimal::serialize)]
    bytes: [u8; 16],
}

impl From<Decimal> for RkyvDecimal {
    fn from(value: Decimal) -> Self {
        Self {
            bytes: value.serialize(),
        }
    }
}

impl From<&Decimal> for RkyvDecimal {
    fn from(value: &Decimal) -> Self {
        Self {
            bytes: value.serialize(),
        }
    }
}

impl From<RkyvDecimal> for Decimal {
    fn from(
        RkyvDecimal {
            bytes,
        }: RkyvDecimal,
    ) -> Self {
        Self::deserialize(bytes)
    }
}
