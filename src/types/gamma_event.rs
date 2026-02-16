use crate::GammaMarket;
use derive_more::{From, Into};
use errgonomic::{ErrVec, handle_bool, handle_iter, handle_opt};
use polymarket_client_sdk::gamma::types::response::Event as GammaEventRaw;
use thiserror::Error;

/// [`GammaEvent`] is a truncation of [`polymarket_client_sdk::gamma::types::response::Event`]
#[derive(From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GammaEvent {
    pub id: String,
    pub slug: String,
    /// NOTE: This Vec is not sorted
    pub markets: Option<Vec<GammaMarket>>,
}

impl GammaEvent {
    /// This function may return multiple opportunities because multiple adjacent markets may exhibit inverted pricing.
    ///
    /// This function assumes that `self` passes [`Self::is_date_cascade`].
    ///
    /// Returns a vec of positive price differences (`prev_yes_price - next_yes_price`).
    pub fn get_time_spread_arbitrage_opportunity(&self) -> Option<Vec<rust_decimal::Decimal>> {
        use itertools::Itertools;
        self.markets.as_ref().map(|markets| {
            markets
                .iter()
                .filter(|market| market.end_date.is_some())
                .sorted_by(|left, right| left.end_date.cmp(&right.end_date))
                .tuple_windows()
                .filter_map(|(prev, next)| {
                    GammaMarket::is_inverted_pricing(prev, next).and_then(|is_inverted| {
                        if is_inverted {
                            prev.yes_price
                                .as_ref()
                                .zip(next.yes_price.as_ref())
                                .and_then(|(prev_yes_price, next_yes_price)| prev_yes_price.checked_sub(*next_yes_price))
                        } else {
                            None
                        }
                    })
                })
                .collect()
        })
    }

    pub fn is_date_cascade(&self) -> Option<bool> {
        self.markets.as_ref().map(|markets| {
            let questions = markets.iter().map(|market| market.question.as_str());
            crate::are_questions_date_cascade(questions)
        })
    }
}

impl TryFrom<GammaEventRaw> for GammaEvent {
    type Error = ConvertGammaEventRawToGammaEventError;

    fn try_from(event: GammaEventRaw) -> Result<Self, Self::Error> {
        use ConvertGammaEventRawToGammaEventError::*;
        let GammaEventRaw {
            id,
            slug,
            markets,
            ..
        } = event;
        handle_bool!(id.trim().is_empty(), EventIdInvalid, id);
        let slug = handle_opt!(slug, SlugMissingInvalid, id);
        let markets = match markets {
            Some(markets) => {
                let markets = handle_iter!(markets.into_iter().map(GammaMarket::try_from), TryFromFailed, id);
                Some(markets)
            }
            None => None,
        };
        Ok(Self {
            id,
            slug,
            markets,
        })
    }
}

#[derive(Error, Debug)]
pub enum ConvertGammaEventRawToGammaEventError {
    #[error("event id is empty")]
    EventIdInvalid { id: String },
    #[error("event slug is missing for id '{id}'")]
    SlugMissingInvalid { id: String },
    #[error("failed to convert '{len}' markets for event '{id}'", len = source.len())]
    TryFromFailed { source: ErrVec<crate::ConvertGammaMarketRawToGammaMarketError>, id: String },
}
