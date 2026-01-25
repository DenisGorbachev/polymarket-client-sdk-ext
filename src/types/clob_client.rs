use crate::{ConvertMarketResponseToMarketError, Market, NEXT_CURSOR_START, NextCursor, TokenId, get_page_stream, is_launched};
use derive_more::{Deref, DerefMut};
use derive_new::new;
use errgonomic::{ErrVec, handle, handle_iter};
use futures::Stream;
use polymarket_client_sdk::clob::types::response::Page;
use std::fmt::Debug;
use thiserror::Error;

#[derive(new, Deref, DerefMut, Default, Clone, Debug)]
pub struct ClobClient {
    pub inner: polymarket_client_sdk::clob::Client,
}

impl ClobClient {
    /// This function returns only launched markets (see [`is_launched`]).
    pub async fn markets(&self, next_cursor: Option<String>) -> Result<Page<Market>, ClobClientMarketsError> {
        use ClobClientMarketsError::*;
        let page = handle!(self.inner.markets(next_cursor.clone()).await, MarketsFailed, next_cursor);
        let Page {
            limit,
            count,
            next_cursor,
            data,
            ..
        } = page;
        let data = handle_iter!(data.into_iter().filter(is_launched).map(Market::try_from), MarketTryFromFailed);
        Ok(Page::builder()
            .data(data)
            .next_cursor(next_cursor)
            .limit(limit)
            .count(count)
            .build())
    }

    pub async fn order_books(&self, _token_ids: impl IntoIterator<Item = &TokenId>) {
        todo!()
    }

    pub fn markets_stream(&self) -> impl Stream<Item = Result<Vec<Market>, ClobClientMarketsError>> + '_ {
        self.get_markets_stream_at_cursor(NEXT_CURSOR_START.into())
    }

    pub fn get_markets_stream_at_cursor(&self, next_cursor: NextCursor) -> impl Stream<Item = Result<Vec<Market>, ClobClientMarketsError>> + '_ {
        get_page_stream(|next_cursor| self.markets(Some(next_cursor)), next_cursor)
    }
}

#[derive(Error, Debug)]
pub enum ClobClientMarketsError {
    #[error("failed to fetch markets page")]
    MarketsFailed { source: polymarket_client_sdk::error::Error, next_cursor: Option<String> },
    #[error("failed to convert '{len}' markets", len = source.len())]
    MarketTryFromFailed { source: ErrVec<ConvertMarketResponseToMarketError> },
}
