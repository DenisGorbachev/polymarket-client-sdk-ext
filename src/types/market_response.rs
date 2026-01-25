use polymarket_client_sdk::clob::types::response::MarketResponse;

/// Some markets haven't been launched.
/// Example: `curl -s "https://clob.polymarket.com/markets?next_cursor=NTAwMA==" | jq '.data[] | select(.market_slug=="arizona-senate-election-2024-will-a-republican-win")'`
/// It is unknown if such non-launched markets may appear in the future
pub fn is_launched(market_response: &MarketResponse) -> bool {
    market_response.question_id.is_some() && market_response.condition_id.is_some()
}
