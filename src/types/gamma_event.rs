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
    pub fn is_date_cascade(&self) -> Option<bool> {
        self.markets.as_ref().and_then(|markets| {
            markets
                .iter()
                .map(|market| market.question.as_deref())
                .collect::<Option<Vec<_>>>()
                .map(crate::are_questions_date_cascade)
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
