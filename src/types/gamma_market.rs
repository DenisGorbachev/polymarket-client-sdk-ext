use crate::{RkyvOffsetDateTime, from_chrono_naive_date};
use derive_more::{From, Into};
use derive_new::new;
use polymarket_client_sdk::gamma::types::response::Market as GammaMarketRaw;
use rkyv::with::Map;
use time::OffsetDateTime;

/// [`GammaMarket`] is a truncation of [`polymarket_client_sdk::gamma::types::response::Market`]
#[derive(new, From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
pub struct GammaMarket {
    #[serde(skip_serializing_if = "Option::is_none")]
    question: Option<String>,
    #[rkyv(with = Map<RkyvOffsetDateTime>)]
    #[serde(skip_serializing_if = "Option::is_none")]
    end_date: Option<OffsetDateTime>,
}

impl GammaMarket {}

// TODO: Fix error handling
impl TryFrom<GammaMarketRaw> for GammaMarket {
    type Error = ();

    fn try_from(raw_gamma_market: GammaMarketRaw) -> Result<Self, Self::Error> {
        let GammaMarketRaw {
            question,
            end_date_iso,
            ..
        } = raw_gamma_market;
        let end_date = end_date_iso.map(from_chrono_naive_date).transpose();
        let end_date = end_date.unwrap();
        Ok(Self {
            question,
            end_date,
        })
    }
}
