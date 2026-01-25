use crate::{NEXT_CURSOR_STOP, NextCursor, NextCursorRef};
use async_stream::stream;
use futures::Stream;
use polymarket_client_sdk::clob::types::response::Page;
use std::fmt::Debug;
use std::future::Future;

pub fn get_page_stream<T: Debug, E, F: Future<Output = Result<Page<T>, E>>>(mut f: impl FnMut(NextCursor) -> F, mut next_cursor: NextCursor) -> impl Stream<Item = Result<Vec<T>, E>> {
    stream! {
        while next_cursor != NEXT_CURSOR_STOP {
            let result = f(next_cursor).await;
            match result {
                Ok(page) => {
                    next_cursor = page.next_cursor;
                    yield Ok(page.data);
                }
                Err(e) => {
                    yield Err(e);
                    break;
                }
            }
        }
    }
}

pub async fn get_page_vec<T, F: Future<Output = Page<T>>>(mut f: impl FnMut(&NextCursorRef) -> F) -> Vec<Vec<T>> {
    let mut output = vec![];
    let mut next_cursor = String::new();
    while next_cursor != NEXT_CURSOR_STOP {
        let payload = f(&next_cursor).await;
        output.push(payload.data);
        next_cursor = payload.next_cursor;
    }
    output
}
