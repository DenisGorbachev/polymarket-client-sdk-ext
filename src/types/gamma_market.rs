use crate::{RkyvOffsetDateTime, from_chrono_naive_date};
use derive_more::{From, Into};
use derive_new::new;
use errgonomic::handle;
use polymarket_client_sdk::gamma::types::response::Market as GammaMarketRaw;
use rkyv::with::Map;
use thiserror::Error;
use time::OffsetDateTime;

/// [`GammaMarket`] is a truncation of [`polymarket_client_sdk::gamma::types::response::Market`]
#[derive(new, From, Into, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct GammaMarket {
    pub question: Option<String>,
    #[rkyv(with = Map<RkyvOffsetDateTime>)]
    pub end_date: Option<OffsetDateTime>,
}

impl GammaMarket {}

impl TryFrom<GammaMarketRaw> for GammaMarket {
    type Error = ConvertGammaMarketRawToGammaMarketError;

    fn try_from(raw_gamma_market: GammaMarketRaw) -> Result<Self, Self::Error> {
        use ConvertGammaMarketRawToGammaMarketError::*;
        let GammaMarketRaw {
            question,
            end_date_iso,
            ..
        } = raw_gamma_market;
        let end_date = match end_date_iso {
            Some(end_date_iso) => {
                let end_date = handle!(from_chrono_naive_date(end_date_iso), FromChronoNaiveDateFailed, end_date_iso);
                Some(end_date)
            }
            None => None,
        };
        Ok(Self {
            question,
            end_date,
        })
    }
}

#[derive(Error, Debug)]
pub enum ConvertGammaMarketRawToGammaMarketError {
    #[error("failed to convert end date '{end_date_iso}'")]
    FromChronoNaiveDateFailed { source: time::error::ComponentRange, end_date_iso: chrono::NaiveDate },
}
