use crate::{BOOLEAN_OUTCOMES, RkyvDecimal, RkyvOffsetDateTime, from_chrono_naive_date};
use derive_more::{From, Into};
use derive_new::new;
use errgonomic::{handle, handle_bool, handle_opt};
use polymarket_client_sdk::gamma::types::response::Market as GammaMarketRaw;
use rkyv::with::Map;
use rust_decimal::Decimal;
use thiserror::Error;
use time::OffsetDateTime;

/// [`GammaMarket`] is a truncation of [`polymarket_client_sdk::gamma::types::response::Market`]
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
        let GammaMarketRaw {
            question,
            outcomes,
            outcome_prices,
            end_date_iso,
            ..
        } = raw_gamma_market;
        let question = handle_opt!(question, QuestionMissing);
        let mut outcome_prices_iter = outcome_prices.unwrap_or_default().into_iter();
        let yes_price = outcome_prices_iter.next();
        let no_price = outcome_prices_iter.next();
        let outcome_prices_rest = outcome_prices_iter.collect::<Vec<_>>();
        handle_bool!(!outcome_prices_rest.is_empty(), UnexpectedOutcomePrices, outcome_prices_rest);
        let end_date = match end_date_iso {
            Some(end_date_iso) => {
                let end_date = handle!(from_chrono_naive_date(end_date_iso), FromChronoNaiveDateFailed, end_date_iso);
                Some(end_date)
            }
            None => None,
        };
        Ok(Self {
            question,
            outcomes,
            yes_price,
            no_price,
            end_date,
        })
    }
}

#[derive(Error, Debug)]
pub enum ConvertGammaMarketRawToGammaMarketError {
    #[error("market question is missing")]
    QuestionMissing {},
    #[error("market has unexpected extra outcome prices: '{len}'", len = outcome_prices_rest.len())]
    UnexpectedOutcomePrices { outcome_prices_rest: Vec<Decimal> },
    #[error("failed to convert end date '{end_date_iso}'")]
    FromChronoNaiveDateFailed { source: time::error::ComponentRange, end_date_iso: chrono::NaiveDate },
}
