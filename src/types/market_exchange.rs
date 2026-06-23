use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MarketExchange {
    Polymarket,
    Opinion,
}

impl Display for MarketExchange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use MarketExchange::*;
        match self {
            Polymarket => f.write_str("polymarket"),
            Opinion => f.write_str("opinion"),
        }
    }
}
