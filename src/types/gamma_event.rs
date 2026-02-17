use crate::{GammaMarket, TimeSpreadArbitrageOpportunity};
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
    pub markets: Vec<GammaMarket>,
    pub is_date_cascade: bool,
}

pub fn is_date_cascade<'a>(markets: impl IntoIterator<Item = &'a GammaMarket>) -> bool {
    let mut markets = markets.into_iter().peekable();
    // this check is needed because otherwise this function will return true for empty markets vec
    if markets.peek().is_none() {
        return false;
    }
    let questions = markets.map(|market| market.question.as_str());
    crate::are_questions_date_cascade(questions)
}

impl GammaEvent {
    pub fn api_url(&self) -> String {
        format!("https://gamma-api.polymarket.com/events/slug/{}", self.slug)
    }

    /// This function may return multiple opportunities because multiple adjacent markets may exhibit inverted pricing.
    ///
    /// Returns all adjacent market pairs where earlier-date YES is priced above later-date YES.
    pub fn get_time_spread_arbitrage_opportunities(&self) -> Option<Vec<TimeSpreadArbitrageOpportunity<'_>>> {
        use itertools::Itertools;
        if !self.is_date_cascade {
            return None;
        }
        let opportunities = self
            .markets
            .iter()
            .filter(|market| market.end_date.is_some())
            .sorted_by(|left, right| left.end_date.cmp(&right.end_date))
            .tuple_windows()
            .filter_map(|(prev, next)| {
                GammaMarket::is_inverted_pricing(prev, next).and_then(|is_inverted| {
                    if is_inverted {
                        Some(TimeSpreadArbitrageOpportunity {
                            event: self,
                            prev,
                            next,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();
        (!opportunities.is_empty()).then_some(opportunities)
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
        let markets = handle_iter!(
            markets
                .unwrap_or_default()
                .into_iter()
                .map(GammaMarket::try_from),
            TryFromFailed,
            id
        );
        let is_date_cascade = is_date_cascade(markets.iter());
        Ok(Self {
            id,
            slug,
            markets,
            is_date_cascade,
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
