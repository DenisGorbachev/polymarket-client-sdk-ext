use crate::{Market, NEXT_CURSOR_START, NextCursor, TokenId, get_page_stream};
use derive_more::{Deref, DerefMut};
use derive_new::new;
use futures::Stream;
use itertools::Itertools;
use polymarket_client_sdk::clob::types::response::Page;
use std::fmt::Debug;
use thiserror::Error;

#[derive(new, Deref, DerefMut, Default, Clone, Debug)]
pub struct ClobClient {
    pub inner: polymarket_client_sdk::clob::Client,
}

impl ClobClient {
    // TODO: Fix error handling
    pub async fn markets(&self, next_cursor: Option<String>) -> Result<Page<Market>, ClobClientMarketsError> {
        // use ClobClientMarketsError::*;
        let page = self.inner.markets(next_cursor).await.unwrap();
        let Page {
            limit,
            count,
            next_cursor,
            data,
            ..
        } = page;
        // TODO: use handle_iter
        let data = data
            .into_iter()
            .map(|market_response| Market::try_from(market_response).unwrap())
            .collect_vec();
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
pub enum ClobClientMarketsError {}
