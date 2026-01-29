use polymarket_client_sdk::clob::types::response::MarketResponse;

/// According to [`crate::CacheCheckCommand`], 2 market responses have question_id
pub fn is_dummy(market_response: &MarketResponse) -> bool {
    market_response.question_id.is_none() || market_response.condition_id.is_none()
}
