use crate::MarketResponsePrecise;
use polymarket_client_sdk::clob::types::response::MarketResponse;

pub trait ShouldDownloadOrderbooks {
    fn should_download_orderbooks(&self) -> bool;
}

impl ShouldDownloadOrderbooks for MarketResponse {
    fn should_download_orderbooks(&self) -> bool {
        self.enable_order_book && self.active && self.accepting_orders && !self.closed && !self.archived
    }
}

impl ShouldDownloadOrderbooks for MarketResponsePrecise {
    fn should_download_orderbooks(&self) -> bool {
        self.enable_order_book && self.active && self.accepting_orders && !self.closed && !self.archived
    }
}
