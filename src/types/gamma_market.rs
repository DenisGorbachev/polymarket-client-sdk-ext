use crate::{BOOLEAN_OUTCOMES, RkyvDecimal, RkyvOffsetDateTime, TIMESTAMP_2023_01_01_00_00_00_Z, from_chrono_naive_date};
use derive_more::{From, Into};
use derive_new::new;
use errgonomic::{handle_bool, handle_opt};
use polymarket_client_sdk::gamma::types::response::Market as GammaMarketRaw;
use rkyv::with::Map;
use rust_decimal::Decimal;
use thiserror::Error;
use time::OffsetDateTime;

/// [`GammaMarket`] is a truncation of [`polymarket_client_sdk::gamma::types::response::Market`] conditional on end_date >= "2023-01-01T00:00:00Z"
#[derive(new, From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GammaMarket {
    pub question: String,
    pub outcomes: Option<Vec<String>>,
    #[serde(with = "rust_decimal::serde::str_option")]
    #[rkyv(with = Map<RkyvDecimal>)]
    pub yes_price: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str_option")]
    #[rkyv(with = Map<RkyvDecimal>)]
    pub no_price: Option<Decimal>,
    #[rkyv(with = Map<RkyvOffsetDateTime>)]
    pub end_date: Option<OffsetDateTime>,
}

impl GammaMarket {
    /// This function assumes that `prev` ends before `next`.
    pub fn is_inverted_pricing(prev: &Self, next: &Self) -> Option<bool> {
        debug_assert!(
            prev.end_date
                .as_ref()
                .zip(next.end_date.as_ref())
                .is_none_or(|(prev_end_date, next_end_date)| prev_end_date < next_end_date)
        );
        prev.outcomes
            .as_ref()
            .zip(next.outcomes.as_ref())
            .and_then(|(prev_outcomes, next_outcomes)| {
                debug_assert_eq!(prev_outcomes.as_slice(), BOOLEAN_OUTCOMES.as_slice());
                debug_assert_eq!(next_outcomes.as_slice(), BOOLEAN_OUTCOMES.as_slice());
                prev.yes_price
                    .as_ref()
                    .zip(next.yes_price.as_ref())
                    .map(|(prev_yes_price, next_yes_price)| prev_yes_price > next_yes_price)
            })
    }
}

impl TryFrom<GammaMarketRaw> for GammaMarket {
    type Error = ConvertGammaMarketRawToGammaMarketError;

    fn try_from(raw_gamma_market: GammaMarketRaw) -> Result<Self, Self::Error> {
        use ConvertGammaMarketRawToGammaMarketError::*;
        let end_date = handle_opt!(raw_gamma_market.end_date, Unsupported, market: raw_gamma_market);
        handle_bool!(end_date.timestamp() < TIMESTAMP_2023_01_01_00_00_00_Z, Unsupported, market: raw_gamma_market);
        let GammaMarketRaw {
            question,
            outcomes,
            outcome_prices,
            end_date_iso,
            ..
        } = raw_gamma_market;
        let mut outcome_prices_iter = outcome_prices.unwrap_or_default().into_iter();
        let yes_price = outcome_prices_iter.next();
        let no_price = outcome_prices_iter.next();
        let outcome_prices_rest = outcome_prices_iter.collect::<Vec<_>>();
        let end_date_result = end_date_iso
            .map(|end_date_iso| {
                from_chrono_naive_date(end_date_iso).map_err(|source| ConvertGammaMarketRawToGammaMarketEndDateError::FromChronoNaiveDateFailed {
                    source,
                    end_date_iso,
                })
            })
            .transpose();
        match (question, end_date_result) {
            (Some(question), Ok(end_date)) if outcome_prices_rest.is_empty() => Ok(Self {
                question,
                outcomes,
                yes_price,
                no_price,
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
    ConversionFailed { question: Option<String>, outcomes: Option<Vec<String>>, yes_price: Option<Decimal>, no_price: Option<Decimal>, outcome_prices_rest: Vec<Decimal>, end_date_result: Result<Option<OffsetDateTime>, ConvertGammaMarketRawToGammaMarketEndDateError> },
}

#[derive(Error, Debug)]
pub enum ConvertGammaMarketRawToGammaMarketEndDateError {
    #[error("failed to convert end date '{end_date_iso}'")]
    FromChronoNaiveDateFailed { source: time::error::ComponentRange, end_date_iso: chrono::NaiveDate },
}
