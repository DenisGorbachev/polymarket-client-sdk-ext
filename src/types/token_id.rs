use alloy::primitives::U256;
use serde::Serializer;

pub type TokenId = U256;

pub fn serialize_as_decimal<S: Serializer>(token_id: &TokenId, serializer: S) -> Result<S::Ok, S::Error> {
    let decimal_string = token_id.to_string();
    serializer.serialize_str(&decimal_string)
}
