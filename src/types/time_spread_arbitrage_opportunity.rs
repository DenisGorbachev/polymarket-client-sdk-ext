use crate::GammaMarket;

#[derive(serde::Serialize, Clone, Debug)]
pub struct TimeSpreadArbitrageOpportunity<'a> {
    pub event_api_url: String,
    pub prev: &'a GammaMarket,
    pub next: &'a GammaMarket,
}
