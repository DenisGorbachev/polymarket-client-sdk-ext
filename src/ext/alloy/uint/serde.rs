use alloy::primitives::Uint;
use core::fmt;
use core::str::FromStr;
use serde::de::{Error, Visitor};
use serde::{Deserializer, Serializer};

pub struct UintAsString;

impl UintAsString {
    pub fn serialize<const BITS: usize, const LIMBS: usize, S>(value: &Uint<BITS, LIMBS>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = value.to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, const BITS: usize, const LIMBS: usize, D>(deserializer: D) -> Result<Uint<BITS, LIMBS>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UintVisitor<const BITS: usize, const LIMBS: usize>;

        impl<'de, const BITS: usize, const LIMBS: usize> Visitor<'de> for UintVisitor<BITS, LIMBS> {
            type Value = Uint<BITS, LIMBS>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a decimal string representing a uint")
            }

            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                Uint::<BITS, LIMBS>::from_str(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(UintVisitor::<BITS, LIMBS>)
    }
}
