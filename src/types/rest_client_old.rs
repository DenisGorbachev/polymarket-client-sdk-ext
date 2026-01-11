use crate::{BookParams, Market, MarketRaw, NEXT_CURSOR_START, NextCursor, Payload, REST_BASE_URL, TokenId, get_page_stream};
use derive_getters::Getters;
use derive_more::{From, Into};
use derive_new::new;
use futures::Stream;
use polymarket_client_sdk::clob::types::response::OrderBookSummaryResponse;
use reqwest::Response;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use url::Url;

#[derive(new, Getters, From, Into, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
pub struct RestClientOld {
    base_url: Url,
}

impl RestClientOld {
    pub fn url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(path);
        url
    }

    pub async fn get_markets(&self, next_cursor: NextCursor) -> reqwest::Result<Payload<Market>> {
        let result = Self::get_payload::<MarketRaw>(self.url("/markets"), next_cursor).await;
        result.map(|payload| {
            let Payload {
                limit,
                count,
                next_cursor,
                data,
            } = payload;
            let data = data
                .into_iter()
                .map(Market::try_from)
                .filter_map(Result::ok)
                .collect::<Vec<Market>>();
            Payload {
                limit,
                count,
                next_cursor,
                data,
            }
        })
    }

    pub async fn get_orderbook_summaries(&self, token_ids: impl IntoIterator<Item = &TokenId>) -> reqwest::Result<Vec<OrderBookSummaryResponse>> {
        let url = self.url("/books");
        let params = token_ids
            .into_iter()
            .map(|token_id| BookParams::from(*token_id))
            .collect::<Vec<BookParams>>();
        Self::post_json(url, &params).await
    }

    pub fn get_markets_stream(&self) -> impl Stream<Item = Result<Vec<Market>, reqwest::Error>> + '_ {
        self.get_markets_stream_at_cursor(NEXT_CURSOR_START.into())
    }

    pub fn get_markets_stream_at_cursor(&self, next_cursor: NextCursor) -> impl Stream<Item = Result<Vec<Market>, reqwest::Error>> + '_ {
        get_page_stream(|next_cursor| self.get_markets(next_cursor), next_cursor)
    }

    // TODO: "If you plan to perform multiple requests, it is best to create a Client and reuse it, taking advantage of keep-alive connection pooling."
    async fn get_payload<T: DeserializeOwned>(mut url: Url, next_cursor: NextCursor) -> reqwest::Result<Payload<T>> {
        url.query_pairs_mut()
            .append_pair("next_cursor", &next_cursor);
        let response = reqwest::get(url).await?;
        Self::parse_response(response).await
    }

    #[cfg(not(feature = "debug"))]
    async fn parse_response<T: DeserializeOwned>(response: Response) -> reqwest::Result<T> {
        response.json().await
    }

    #[cfg(feature = "debug")]
    async fn parse_response<T: DeserializeOwned>(response: Response) -> reqwest::Result<T> {
        use std::fs::File;
        use std::io::Write;
        let full = response.text().await?;
        File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open("/tmp/response.json")
            .unwrap()
            .write_all(full.as_bytes())
            .unwrap();
        let data = serde_json::from_slice(full.as_bytes()).unwrap();
        Ok(data)
    }

    async fn post_json<Params: Serialize + ?Sized, T: DeserializeOwned>(url: Url, params: &Params) -> reqwest::Result<T> {
        let client = reqwest::Client::new();
        let request = client.post(url).json(params).build()?;
        let response = client.execute(request).await?;
        Self::parse_response(response).await
    }
}

impl Default for RestClientOld {
    fn default() -> Self {
        Self {
            base_url: REST_BASE_URL.clone(),
        }
    }
}
