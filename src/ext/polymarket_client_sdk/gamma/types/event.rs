use polymarket_client_sdk::gamma::types::response::Event as GammaEventRaw;

mod slug_is_none;

use crate::option_date_time_is_fresh;
pub use slug_is_none::*;

pub fn gamma_event_raw_is_fresh(event: &GammaEventRaw) -> bool {
    let is_event_fresh = option_date_time_is_fresh(event.end_date);
    let is_event_markets_fresh = event
        .markets
        .as_ref()
        .map(|markets| {
            markets
                .iter()
                .all(|m| option_date_time_is_fresh(m.end_date))
        })
        .unwrap_or_default();
    is_event_fresh && is_event_markets_fresh
}
