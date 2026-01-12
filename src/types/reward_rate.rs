use crate::Amount;
use alloy::primitives::Address;
use derive_more::{From, Into};
use derive_new::new;
use polymarket_client_sdk::clob::types::response::RewardRate as RewardRateRaw;
use serde::{Deserialize, Serialize};

#[derive(new, From, Into, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RewardRate {
    pub asset_address: Address,
    pub rewards_daily_rate: Amount,
}

impl RewardRate {}

impl From<RewardRateRaw> for RewardRate {
    fn from(value: RewardRateRaw) -> Self {
        let RewardRateRaw {
            asset_address,
            rewards_daily_rate,
            ..
        } = value;
        Self {
            asset_address,
            rewards_daily_rate,
        }
    }
}

impl From<RewardRate> for RewardRateRaw {
    fn from(value: RewardRate) -> Self {
        let RewardRate {
            asset_address,
            rewards_daily_rate,
        } = value;
        RewardRateRaw::builder()
            .asset_address(asset_address)
            .rewards_daily_rate(rewards_daily_rate)
            .build()
    }
}
