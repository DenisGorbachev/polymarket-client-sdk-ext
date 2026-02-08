use crate::{Amount, RewardRate, RkyvDecimal};
use derive_more::{From, Into};
use derive_new::new;
use polymarket_client_sdk::clob::types::response::Rewards as RewardsRaw;

/// Using our own `Rewards` to gain `Eq` and `Hash`
#[derive(new, From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Rewards {
    pub rates: Vec<RewardRate>,
    #[rkyv(with = RkyvDecimal)]
    #[serde(with = "rust_decimal::serde::str")]
    pub min_size: Amount,
    #[rkyv(with = RkyvDecimal)]
    #[serde(with = "rust_decimal::serde::str")]
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
