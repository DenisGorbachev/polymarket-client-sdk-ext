use futures::StreamExt;
use polymarket_api::{Market, RestClientOld, NEXT_CURSOR_START};
use std::env;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use tokio::pin;

mod common;

pub const MAX_PAGES_DEFAULT: &str = "4";

#[tokio::test]
async fn test_markets() {
    env_logger::init();
    let next_cursor = env::var("NEXT_CURSOR").unwrap_or(NEXT_CURSOR_START.into());
    let max_pages_string = env::var("MAX_PAGES").unwrap_or(MAX_PAGES_DEFAULT.into());
    let markets_filename = env::var("MARKETS_FILENAME").ok();
    let max_pages = usize::from_str(&max_pages_string).unwrap();
    let client = RestClientOld::default();
    let markets_stream = client.get_markets_stream_at_cursor(next_cursor);
    let markets_stream = markets_stream.take(max_pages);
    pin!(markets_stream);
    let markets_results = markets_stream
        .collect::<Vec<reqwest::Result<Vec<Market>>>>()
        .await;
    let markets_pages = markets_results
        .into_iter()
        .collect::<reqwest::Result<Vec<Vec<Market>>>>()
        .unwrap();
    let markets = markets_pages.into_iter().flatten().collect::<Vec<Market>>();
    if let Some(markets_filename) = markets_filename {
        let markets_dump = serde_json::to_string(&markets).unwrap();
        File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(markets_filename)
            .unwrap()
            .write_all(markets_dump.as_bytes())
            .unwrap();
    }
    let donald_trump = markets
        .iter()
        .find(|market| market.market_slug == "will-donald-trump-jr-win-the-us-2024-republican-presidential-nomination");
    assert!(donald_trump.is_some());
}

// pub fn collect_all() ->  {
//     todo!()
// }
