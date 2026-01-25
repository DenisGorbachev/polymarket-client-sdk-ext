use alloy::primitives::U256;
use serde::Serializer;

pub type TokenId = U256;

pub fn to_fjall_key_from_token_id(token_id: TokenId) -> [u8; 32] {
    token_id.to_be_bytes::<32>()
}

pub fn serialize_as_decimal<S: Serializer>(token_id: &TokenId, serializer: S) -> Result<S::Ok, S::Error> {
    let decimal_string = token_id.to_string();
    serializer.serialize_str(&decimal_string)
}
