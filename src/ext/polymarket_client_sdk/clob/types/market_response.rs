use polymarket_client_sdk::clob::types::response::MarketResponse;

/// According to [`crate::CacheCheckCommand`] and [`crate::QuestionIdIsNoneIffConditionIdIsNone`], only 2 market responses have question_id and condition_id with different variants, so most market responses have both the question_id and condition_id with Some variant
pub fn is_dummy(market_response: &MarketResponse) -> bool {
    market_response.question_id.is_none() || market_response.condition_id.is_none()
}
