use crate::{GammaEvent, GammaMarket};

#[derive(Copy, Clone, Debug)]
pub struct TimeSpreadArbitrageOpportunity<'a> {
    pub event: &'a GammaEvent,
    pub prev: &'a GammaMarket,
    pub next: &'a GammaMarket,
}
