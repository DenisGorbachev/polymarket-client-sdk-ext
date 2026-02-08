use crate::{Amount, Price};
use derive_more::{From, Into};
use polymarket_client_sdk::clob::types::response::OrderSummary;
use serde::{Deserialize, Serialize};

#[derive(From, Into, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[serde(deny_unknown_fields)]
pub struct Level {
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Price,
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Amount,
}

impl Level {}

impl From<(&Price, &Amount)> for Level {
    fn from((price, size): (&Price, &Amount)) -> Self {
        Self {
            price: *price,
            size: *size,
        }
    }
}

impl From<OrderSummary> for Level {
    fn from(summary: OrderSummary) -> Self {
        let OrderSummary {
            price,
            size,
            ..
        } = summary;
        Self {
            price,
            size,
        }
    }
}
