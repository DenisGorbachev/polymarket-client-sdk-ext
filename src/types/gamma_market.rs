use crate::{BOOLEAN_OUTCOMES, RkyvDecimal, RkyvOffsetDateTime, from_chrono_naive_date, gamma_market_raw_is_fresh};
use derive_more::{From, Into};
use derive_new::new;
use errgonomic::handle_bool;
use polymarket_client_sdk::gamma::types::response::Market as GammaMarketRaw;
use rkyv::with::Map;
use rust_decimal::Decimal;
use thiserror::Error;
use time::OffsetDateTime;
use time::error::ComponentRange;

/// [`GammaMarket`] is a truncation of [`polymarket_client_sdk::gamma::types::response::Market`] conditional on end_date >= "2023-01-01T00:00:00Z"
#[derive(new, From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GammaMarket {
    pub question: String,

    pub outcomes: Option<Vec<String>>,

    #[serde(with = "rust_decimal::serde::str_option")]
    #[rkyv(with = Map<RkyvDecimal>)]
    pub price_yes: Option<Decimal>,

    #[serde(with = "rust_decimal::serde::str_option")]
    #[rkyv(with = Map<RkyvDecimal>)]
    pub price_no: Option<Decimal>,

    #[rkyv(with = RkyvOffsetDateTime)]
    pub end_date: OffsetDateTime,
}

impl GammaMarket {
    /// This function assumes that `prev` ends before `next`.
    pub fn is_inverted_pricing(prev: &Self, next: &Self) -> Option<bool> {
        debug_assert!(prev.end_date < next.end_date);
        prev.outcomes
            .as_ref()
            .zip(next.outcomes.as_ref())
            .and_then(|(prev_outcomes, next_outcomes)| {
                debug_assert_eq!(prev_outcomes.as_slice(), BOOLEAN_OUTCOMES.as_slice());
                debug_assert_eq!(next_outcomes.as_slice(), BOOLEAN_OUTCOMES.as_slice());
                prev.price_yes
                    .as_ref()
                    .zip(next.price_yes.as_ref())
                    .map(|(prev_yes_price, next_yes_price)| prev_yes_price > next_yes_price)
            })
    }
}

impl TryFrom<GammaMarketRaw> for GammaMarket {
    type Error = ConvertGammaMarketRawToGammaMarketError;

    fn try_from(market: GammaMarketRaw) -> Result<Self, Self::Error> {
        use ConvertGammaMarketRawToGammaMarketError::*;
        handle_bool!(!gamma_market_raw_is_fresh(&market), Unsupported, market);
        let GammaMarketRaw {
            question,
            outcomes,
            outcome_prices,
            end_date_iso,
            ..
        } = market;
        let mut outcome_prices_iter = outcome_prices.unwrap_or_default().into_iter();
        let yes_price = outcome_prices_iter.next();
        let no_price = outcome_prices_iter.next();
        let outcome_prices_rest = outcome_prices_iter.collect::<Vec<_>>();
        let end_date_result = end_date_iso.map(from_chrono_naive_date).transpose();
        match (question, end_date_result) {
            (Some(question), Ok(Some(end_date))) if outcome_prices_rest.is_empty() => Ok(Self {
                question,
                outcomes,
                price_yes: yes_price,
                price_no: no_price,
                end_date,
            }),
            (question, end_date_result) => Err(ConversionFailed {
                question,
                outcomes,
                yes_price,
                no_price,
                outcome_prices_rest,
                end_date_result,
            }),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConvertGammaMarketRawToGammaMarketError {
    #[error("old gamma market not supported")]
    Unsupported { market: Box<GammaMarketRaw> },
    #[error("failed to convert gamma market")]
    ConversionFailed { question: Option<String>, outcomes: Option<Vec<String>>, yes_price: Option<Decimal>, no_price: Option<Decimal>, outcome_prices_rest: Vec<Decimal>, end_date_result: Result<Option<OffsetDateTime>, ComponentRange> },
}
