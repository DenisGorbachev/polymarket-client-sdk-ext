use crate::{NEXT_CURSOR_STOP, NextCursor, Payload};
use futures::Stream;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::pin::{Pin, pin};
use std::task::Poll::*;
use std::task::{Context, Poll};
use url::Url;

pub struct PayloadIterator<T> {
    url: Url,
    next_cursor: String,
    #[allow(dead_code)]
    future: Option<Box<dyn Future<Output = Payload<T>>>>,
}

impl<T> PayloadIterator<T> {
    pub fn new(url: impl Into<Url>) -> Self {
        Self::new_with_cursor(url, "")
    }

    pub fn new_with_cursor(url: impl Into<Url>, next_cursor: impl Into<NextCursor>) -> Self {
        Self {
            url: url.into(),
            next_cursor: next_cursor.into(),
            future: None,
        }
    }
}

impl<T> Stream for PayloadIterator<T>
where
    T: DeserializeOwned + Send + 'static,
{
    type Item = Result<Vec<T>, reqwest::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.is_final() {
            Ready(None)
        } else {
            let mut url = self.url.clone();
            url.query_pairs_mut()
                .append_pair("next_cursor", &self.next_cursor);
            let response_result = match pin!(reqwest::get(url)).poll(cx) {
                Ready(resp) => resp,
                Pending => return Pending,
            };
            let response = match response_result {
                Ok(response) => response,
                Err(err) => return Ready(Some(Err(err))),
            };
            let payload_result: reqwest::Result<Payload<T>> = match pin!(response.json()).poll(cx) {
                Ready(result) => result,
                Pending => return Pending,
            };
            let payload = match payload_result {
                Ok(payload) => payload,
                Err(err) => return Ready(Some(Err(err))),
            };

            self.next_cursor = payload.next_cursor;

            Ready(Some(Ok(payload.data)))
        }
    }
}

impl<T> PayloadIterator<T> {
    pub fn is_final(&self) -> bool {
        self.next_cursor == NEXT_CURSOR_STOP
    }
}
