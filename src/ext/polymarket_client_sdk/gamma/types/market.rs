use crate::option_date_time_is_fresh;
use polymarket_client_sdk::gamma::types::response::Market as GammaMarketRaw;

pub fn gamma_market_raw_is_fresh(market: &GammaMarketRaw) -> bool {
    option_date_time_is_fresh(market.end_date)
}
