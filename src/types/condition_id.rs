use alloy::primitives::B256;

pub type ConditionId = B256;

pub fn to_fjall_key_from_condition_id(condition_id: ConditionId) -> [u8; 32] {
    condition_id.0
}
