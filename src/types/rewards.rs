use crate::{Amount, RewardRate};
use derive_more::{From, Into};
use derive_new::new;
use polymarket_client_sdk::clob::types::response::Rewards as RewardsRaw;
use serde::{Deserialize, Serialize};

/// Using our own `Rewards` to gain `Eq` and `Hash`
#[derive(new, From, Into, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Rewards {
    pub rates: Vec<RewardRate>,
    pub min_size: Amount,
    pub max_spread: Amount,
}

impl Rewards {}

impl From<RewardsRaw> for Rewards {
    fn from(value: RewardsRaw) -> Self {
        let RewardsRaw {
            rates,
            min_size,
            max_spread,
            ..
        } = value;
        let rates = rates.into_iter().map(From::from).collect();
        Self {
            rates,
            min_size,
            max_spread,
        }
    }
}

impl From<Rewards> for RewardsRaw {
    fn from(value: Rewards) -> Self {
        let Rewards {
            rates,
            min_size,
            max_spread,
        } = value;
        let rates = rates.into_iter().map(Into::into).collect();
        RewardsRaw::builder()
            .rates(rates)
            .min_size(min_size)
            .max_spread(max_spread)
            .build()
    }
}
