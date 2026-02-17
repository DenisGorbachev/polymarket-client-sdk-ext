use crate::{ConvertGammaMarketRawToGammaMarketError, GammaMarket, TimeSpreadArbitrageOpportunity, are_questions_date_cascade, gamma_event_raw_is_fresh};
use derive_more::{From, Into};
use errgonomic::{ErrVec, handle_bool, partition_result};
use polymarket_client_sdk::gamma::types::response::Event as GammaEventRaw;
use thiserror::Error;

/// [`GammaEvent`] is a truncation of [`polymarket_client_sdk::gamma::types::response::Event`] conditional on end_date >= "2023-01-01T00:00:00Z"
#[derive(From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GammaEvent {
    pub id: String,
    pub slug: String,
    /// NOTE: This Vec is not sorted
    pub markets: Vec<GammaMarket>,
    pub is_date_cascade: Option<bool>,
}

pub fn is_date_cascade(markets: &[GammaMarket]) -> Option<bool> {
    if markets.len() < 2 {
        return None;
    }
    let questions = markets.iter().map(|market| market.question.as_str());
    Some(are_questions_date_cascade(questions))
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
        if !self.is_date_cascade.unwrap_or_default() {
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
        handle_bool!(!gamma_event_raw_is_fresh(&event), Unsupported, event);
        let GammaEventRaw {
            id,
            slug,
            markets,
            ..
        } = event;
        let markets_result = match partition_result(
            markets
                .unwrap_or_default()
                .into_iter()
                .map(GammaMarket::try_from),
        ) {
            Ok(markets) => Ok(markets),
            Err(source) => Err(source.into()),
        };
        let is_event_id_empty = id.trim().is_empty();
        match (is_event_id_empty, slug, markets_result) {
            (false, Some(slug), Ok(markets)) => {
                let is_date_cascade = is_date_cascade(&markets);
                Ok(Self {
                    id,
                    slug,
                    markets,
                    is_date_cascade,
                })
            }
            (is_event_id_empty, slug, markets_result) => Err(ConversionFailed {
                id,
                slug,
                markets_result,
                is_event_id_empty,
            }),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConvertGammaEventRawToGammaEventError {
    #[error("old gamma event not supported")]
    Unsupported { event: Box<GammaEventRaw> },
    #[error("failed to convert gamma event")]
    ConversionFailed { id: String, slug: Option<String>, markets_result: Result<Vec<GammaMarket>, ErrVec<ConvertGammaMarketRawToGammaMarketError>>, is_event_id_empty: bool },
}
