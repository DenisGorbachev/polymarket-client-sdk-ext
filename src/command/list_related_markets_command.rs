use crate::{MarketExchange, MarketRelation, MarketRelationInfo, OpinionMarket, OpinionMarketPage, RelatedMarketsFormat};
use async_stream::stream;
use core::num::TryFromIntError;
use errgonomic::{handle, handle_opt};
use futures::{Stream, StreamExt};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::MarketsRequest;
use polymarket_client_sdk::gamma::types::response::Market as PolymarketMarket;
use rustc_hash::FxHashMap;
use std::env::{VarError, var};
use std::fs::{File, create_dir_all};
use std::io::{self, BufRead, BufReader, Write, stdout};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::process::ExitCode;
use std::time::Duration;
use tempfile::{NamedTempFile, PersistError};
use thiserror::Error;
use url::{ParseError as UrlParseError, Url};

const LIST_RELATED_MARKETS_CACHE_DIR: &str = ".cache/list_related_markets";
const OPINION_API_KEY_ENV: &str = "OPINION_API_KEY";
const OPINION_MARKETS_URL: &str = "https://api.opinion.trade/api/v1/markets";
const OPINION_PER_PAGE: u32 = 100;
const OPINION_PER_PAGE_USIZE: usize = 100;
const POLYMARKET_PAGE_SIZE: i32 = 1000;
const POLYMARKET_PAGE_SIZE_USIZE: usize = 1000;
const REQUEST_TIMEOUT_SECONDS: u64 = 30;
const RELATION_EQUIVALENT: &str = "a == b";

type OpinionMarketsByQuestion = FxHashMap<String, Vec<MarketRelationInfo>>;

#[derive(clap::Parser, Clone, Debug)]
pub struct ListRelatedMarketsCommand {
    #[arg(long, value_enum, default_value_t = RelatedMarketsFormat::Json)]
    pub format: RelatedMarketsFormat,

    #[arg(long)]
    pub offset: Option<usize>,

    #[arg(long)]
    pub limit: Option<NonZeroUsize>,
}

impl ListRelatedMarketsCommand {
    pub async fn run(self) -> Result<ExitCode, ListRelatedMarketsCommandRunError> {
        use ListRelatedMarketsCommandRunError::*;
        let Self {
            format,
            offset,
            limit,
        } = self;
        let offset = offset.unwrap_or(0);
        let limit = limit.map(NonZeroUsize::get);
        let cache_path = Self::cache_path_for_query(offset, limit);
        let mut writer = stdout().lock();
        if cache_path.exists() {
            handle!(Self::write_relations_from_cache(&cache_path, format, &mut writer), WriteRelationsFromCacheFailed, cache_path);
        } else {
            handle!(Self::write_relations_from_network_and_cache(&cache_path, offset, limit, format, &mut writer).await, WriteRelationsFromNetworkAndCacheFailed, cache_path, offset, limit);
        }
        Ok(ExitCode::SUCCESS)
    }

    fn cache_path_for_query(offset: usize, limit: Option<usize>) -> PathBuf {
        let file_name = match limit {
            Some(limit) => format!("offset-{offset}.limit-{limit}.jsonl"),
            None => format!("offset-{offset}.limit-all.jsonl"),
        };
        PathBuf::from(LIST_RELATED_MARKETS_CACHE_DIR).join(file_name)
    }

    fn write_relations_from_cache(cache_path: &Path, format: RelatedMarketsFormat, writer: &mut impl Write) -> Result<(), ListRelatedMarketsCommandWriteRelationsFromCacheError> {
        use ListRelatedMarketsCommandWriteRelationsFromCacheError::*;
        let file = handle!(File::open(cache_path), OpenFailed, cache_path: cache_path.to_path_buf());
        let reader = BufReader::new(file);
        let mut relation_results = reader.lines().enumerate().map(|(line_index, line_result)| {
            use ListRelatedMarketsCommandWriteRelationsFromCacheError::*;
            let line = handle!(line_result, ReadLineFailed, line_index);
            let line_number = handle_opt!(line_index.checked_add(1), LineNumberCheckedAddFailed, line_index);
            let relation = handle!(serde_json::from_str::<MarketRelation>(&line), DeserializeLineFailed, line, line_number);
            Ok(relation)
        });
        relation_results.try_for_each(|relation| {
            let relation = match relation {
                Ok(relation) => relation,
                Err(error) => return Err(error),
            };
            handle!(Self::write_relation_line(&relation, format, writer), WriteRelationLineFailed);
            Ok(())
        })
    }

    async fn write_relations_from_network_and_cache(cache_path: &Path, offset: usize, limit: Option<usize>, format: RelatedMarketsFormat, writer: &mut impl Write) -> Result<(), ListRelatedMarketsCommandWriteRelationsFromNetworkAndCacheError> {
        use ListRelatedMarketsCommandWriteRelationsFromNetworkAndCacheError::*;
        let cache_dir = handle_opt!(cache_path.parent(), CacheParentNotFound, cache_path: cache_path.to_path_buf());
        handle!(create_dir_all(cache_dir), CreateDirAllFailed, cache_dir: cache_dir.to_path_buf());
        let mut temp_file = handle!(NamedTempFile::new_in(cache_dir), CreateTempFileFailed, cache_dir: cache_dir.to_path_buf());
        let relation_stream = handle!(Self::network_relation_stream().await, NetworkRelationStreamFailed);
        let mut relation_stream = pin!(relation_stream);
        let mut skipped = 0usize;
        let mut emitted = 0usize;

        while let Some(relation_result) = relation_stream.next().await {
            let relation = handle!(relation_result, NetworkRelationItemFailed);
            if skipped < offset {
                skipped = handle_opt!(skipped.checked_add(1), SkippedCheckedAddFailed, skipped);
                continue;
            }
            if let Some(limit) = limit
                && emitted >= limit
            {
                break;
            }
            handle!(Self::write_relation_line(&relation, format, writer), WriteRelationLineFailed);
            handle!(Self::write_relation_to_cache_line(&mut temp_file, &relation), WriteRelationToCacheLineFailed);
            emitted = handle_opt!(emitted.checked_add(1), EmittedCheckedAddFailed, emitted);
        }

        handle!(temp_file.flush(), FlushFailed);
        let _persisted = handle!(temp_file.persist(cache_path), PersistFailed, cache_path: cache_path.to_path_buf());
        Ok(())
    }

    fn write_relation_to_cache_line(temp_file: &mut NamedTempFile, relation: &MarketRelation) -> Result<(), ListRelatedMarketsCommandWriteRelationToCacheLineError> {
        use ListRelatedMarketsCommandWriteRelationToCacheLineError::*;
        handle!(serde_json::to_writer(&mut *temp_file, relation), SerializeFailed);
        handle!(temp_file.write_all(b"\n"), WriteNewlineFailed);
        Ok(())
    }

    async fn network_relation_stream() -> Result<impl Stream<Item = Result<MarketRelation, ListRelatedMarketsCommandNetworkRelationStreamItemError>>, ListRelatedMarketsCommandNetworkRelationStreamError> {
        use ListRelatedMarketsCommandNetworkRelationStreamError::*;
        let opinion_markets_by_question = handle!(Self::fetch_opinion_markets_by_question().await, FetchOpinionMarketsByQuestionFailed);
        Ok(Self::network_relation_stream_from_opinion_markets(opinion_markets_by_question))
    }

    fn network_relation_stream_from_opinion_markets(opinion_markets_by_question: OpinionMarketsByQuestion) -> impl Stream<Item = Result<MarketRelation, ListRelatedMarketsCommandNetworkRelationStreamItemError>> {
        stream! {
            use ListRelatedMarketsCommandNetworkRelationStreamItemError::*;
            let client = GammaClient::default();
            let mut offset: i32 = 0;
            loop {
                let request = MarketsRequest::builder()
                    .limit(POLYMARKET_PAGE_SIZE)
                    .offset(offset)
                    .order("id".to_string())
                    .ascending(true)
                    .build();
                let markets_result = client.markets(&request).await;
                let markets = match markets_result {
                    Ok(markets) => markets,
                    Err(source) => {
                        yield Err(MarketsFailed {
                            source,
                            request: Box::new(request),
                        });
                        break;
                    }
                };
                if markets.is_empty() {
                    break;
                }
                let market_count = markets.len();
                for polymarket_market in markets {
                    let polymarket_market_info_opt = Self::polymarket_market_to_relation_info(polymarket_market);
                    if let Some(polymarket_market_info) = polymarket_market_info_opt {
                        let normalized_question = Self::normalize_question(polymarket_market_info.question.as_str());
                        let opinion_markets_opt = opinion_markets_by_question.get(&normalized_question);
                        if let Some(opinion_markets) = opinion_markets_opt {
                            for opinion_market in opinion_markets.iter() {
                                let relation = MarketRelation {
                                    a: opinion_market.clone(),
                                    b: polymarket_market_info.clone(),
                                    relation: RELATION_EQUIVALENT.to_string(),
                                };
                                yield Ok(relation);
                            }
                        }
                    }
                }

                let market_count_i32 = match i32::try_from(market_count) {
                    Ok(market_count_i32) => market_count_i32,
                    Err(source) => {
                        yield Err(TryFromMarketCountFailed {
                            source,
                            market_count,
                        });
                        break;
                    }
                };
                offset = match offset.checked_add(market_count_i32) {
                    Some(offset) => offset,
                    None => {
                        yield Err(OffsetCheckedAddFailed {
                            offset,
                            market_count_i32,
                        });
                        break;
                    }
                };
                if market_count < POLYMARKET_PAGE_SIZE_USIZE {
                    break;
                }
            }
        }
    }

    async fn fetch_opinion_markets_by_question() -> Result<OpinionMarketsByQuestion, ListRelatedMarketsCommandFetchOpinionMarketsByQuestionError> {
        use ListRelatedMarketsCommandFetchOpinionMarketsByQuestionError::*;
        let api_key = handle!(var(OPINION_API_KEY_ENV), MissingApiKeyFailed);
        let client = handle!(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
                .build(),
            BuildClientFailed
        );
        let mut page: u32 = 1;
        let mut output = OpinionMarketsByQuestion::default();
        loop {
            let request_url = handle!(Self::opinion_market_page_url(page), OpinionMarketPageUrlFailed, page);
            let response = handle!(
                client
                    .get(request_url)
                    .header("x-api-key", api_key.as_str())
                    .send()
                    .await,
                SendFailed,
                page
            );
            let response = handle!(response.error_for_status(), ErrorForStatusFailed, page);
            let page_payload = handle!(response.json::<OpinionMarketPage>().await, JsonFailed, page);
            let OpinionMarketPage {
                data,
            } = page_payload;
            let market_count = data.len();
            let relation_infos = data
                .into_iter()
                .flat_map(OpinionMarket::into_market_relation_infos);
            output = relation_infos.fold(output, |mut output, relation_info| {
                let normalized_question = Self::normalize_question(relation_info.question.as_str());
                output
                    .entry(normalized_question)
                    .or_default()
                    .push(relation_info);
                output
            });
            if market_count < OPINION_PER_PAGE_USIZE {
                break;
            }
            page = handle_opt!(page.checked_add(1), PageCheckedAddFailed, page);
        }
        Ok(output)
    }

    fn opinion_market_page_url(page: u32) -> Result<Url, ListRelatedMarketsCommandOpinionMarketPageUrlError> {
        use ListRelatedMarketsCommandOpinionMarketPageUrlError::*;
        let mut url = handle!(Url::parse(OPINION_MARKETS_URL), ParseFailed);
        let per_page_string = OPINION_PER_PAGE.to_string();
        let page_string = page.to_string();
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("per_page", per_page_string.as_str());
            query_pairs.append_pair("page", page_string.as_str());
        }
        Ok(url)
    }

    fn polymarket_market_to_relation_info(market: PolymarketMarket) -> Option<MarketRelationInfo> {
        use MarketExchange::*;
        let PolymarketMarket {
            id,
            question,
            slug,
            ..
        } = market;
        question.map(|question| MarketRelationInfo {
            exchange: Polymarket,
            id,
            slug,
            question,
        })
    }

    fn normalize_question(input: &str) -> String {
        let cleaned = input
            .chars()
            .map(|char| {
                if char.is_ascii_alphanumeric() || char.is_ascii_whitespace() {
                    char.to_ascii_lowercase()
                } else {
                    ' '
                }
            })
            .collect::<String>();
        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn write_relation_line(relation: &MarketRelation, format: RelatedMarketsFormat, writer: &mut impl Write) -> Result<(), ListRelatedMarketsCommandWriteRelationLineError> {
        use ListRelatedMarketsCommandWriteRelationLineError::*;
        use RelatedMarketsFormat::*;
        match format {
            Json => {
                handle!(serde_json::to_writer(&mut *writer, relation), SerializeFailed);
                handle!(writer.write_all(b"\n"), WriteNewlineFailed);
            }
            Short => {
                let line = Self::short_relation_line(relation);
                handle!(writer.write_all(line.as_bytes()), WriteLineFailed);
                handle!(writer.write_all(b"\n"), WriteNewlineFailed);
            }
        }
        Ok(())
    }

    fn short_relation_line(relation: &MarketRelation) -> String {
        let MarketRelation {
            a,
            b,
            relation,
        } = relation;
        format!("[{a_exchange}:{a_id}] '{a_question}' -> [{b_exchange}:{b_id}] '{b_question}' ({relation})", a_exchange = a.exchange, a_id = a.id, a_question = a.question, b_exchange = b.exchange, b_id = b.id, b_question = b.question, relation = relation,)
    }
}

#[derive(Error, Debug)]
pub enum ListRelatedMarketsCommandRunError {
    #[error("failed to write related markets from cache at '{cache_path}'")]
    WriteRelationsFromCacheFailed { source: ListRelatedMarketsCommandWriteRelationsFromCacheError, cache_path: PathBuf },
    #[error("failed to write related markets from network and cache at '{cache_path}' (offset: {offset}, limit: {limit:?})")]
    WriteRelationsFromNetworkAndCacheFailed { source: Box<ListRelatedMarketsCommandWriteRelationsFromNetworkAndCacheError>, cache_path: PathBuf, offset: usize, limit: Option<usize> },
}

#[derive(Error, Debug)]
pub enum ListRelatedMarketsCommandWriteRelationsFromCacheError {
    #[error("failed to open cache file at '{cache_path}'")]
    OpenFailed { source: io::Error, cache_path: PathBuf },
    #[error("failed to read line at index {line_index}")]
    ReadLineFailed { source: io::Error, line_index: usize },
    #[error("failed to increment line number for line index {line_index}")]
    LineNumberCheckedAddFailed { line_index: usize },
    #[error("failed to deserialize cache line {line_number}")]
    DeserializeLineFailed { source: serde_json::Error, line: String, line_number: usize },
    #[error("failed to write relation line")]
    WriteRelationLineFailed { source: ListRelatedMarketsCommandWriteRelationLineError },
}

#[derive(Error, Debug)]
pub enum ListRelatedMarketsCommandWriteRelationsFromNetworkAndCacheError {
    #[error("cache path parent not found for '{cache_path}'")]
    CacheParentNotFound { cache_path: PathBuf },
    #[error("failed to create cache dir '{cache_dir}'")]
    CreateDirAllFailed { source: io::Error, cache_dir: PathBuf },
    #[error("failed to create temp file in '{cache_dir}'")]
    CreateTempFileFailed { source: io::Error, cache_dir: PathBuf },
    #[error("failed to initialize network relation stream")]
    NetworkRelationStreamFailed { source: Box<ListRelatedMarketsCommandNetworkRelationStreamError> },
    #[error("failed to read item from network relation stream")]
    NetworkRelationItemFailed { source: ListRelatedMarketsCommandNetworkRelationStreamItemError },
    #[error("failed to increment skipped count {skipped}")]
    SkippedCheckedAddFailed { skipped: usize },
    #[error("failed to write relation line")]
    WriteRelationLineFailed { source: ListRelatedMarketsCommandWriteRelationLineError },
    #[error("failed to write relation cache line")]
    WriteRelationToCacheLineFailed { source: ListRelatedMarketsCommandWriteRelationToCacheLineError },
    #[error("failed to increment emitted count {emitted}")]
    EmittedCheckedAddFailed { emitted: usize },
    #[error("failed to flush relation cache file")]
    FlushFailed { source: io::Error },
    #[error("failed to persist relation cache to '{cache_path}'")]
    PersistFailed { source: PersistError, cache_path: PathBuf },
}

#[derive(Error, Debug)]
pub enum ListRelatedMarketsCommandWriteRelationToCacheLineError {
    #[error("failed to serialize related market relation")]
    SerializeFailed { source: serde_json::Error },
    #[error("failed to write cache newline")]
    WriteNewlineFailed { source: io::Error },
}

#[derive(Error, Debug)]
pub enum ListRelatedMarketsCommandNetworkRelationStreamError {
    #[error("failed to fetch opinion markets")]
    FetchOpinionMarketsByQuestionFailed { source: Box<ListRelatedMarketsCommandFetchOpinionMarketsByQuestionError> },
}

#[derive(Error, Debug)]
pub enum ListRelatedMarketsCommandNetworkRelationStreamItemError {
    #[error("failed to fetch polymarket markets for request '{request:?}'")]
    MarketsFailed { source: polymarket_client_sdk::error::Error, request: Box<MarketsRequest> },
    #[error("failed to convert market count '{market_count}' to i32")]
    TryFromMarketCountFailed { source: TryFromIntError, market_count: usize },
    #[error("failed to increment polymarket offset {offset} by {market_count_i32}")]
    OffsetCheckedAddFailed { offset: i32, market_count_i32: i32 },
}

#[derive(Error, Debug)]
pub enum ListRelatedMarketsCommandFetchOpinionMarketsByQuestionError {
    #[error("failed to read environment variable 'OPINION_API_KEY'")]
    MissingApiKeyFailed { source: VarError },
    #[error("failed to build opinion http client")]
    BuildClientFailed { source: reqwest::Error },
    #[error("failed to build opinion market URL for page {page}")]
    OpinionMarketPageUrlFailed { source: ListRelatedMarketsCommandOpinionMarketPageUrlError, page: u32 },
    #[error("failed to fetch opinion market page {page}")]
    SendFailed { source: reqwest::Error, page: u32 },
    #[error("opinion market page {page} returned non-success status")]
    ErrorForStatusFailed { source: reqwest::Error, page: u32 },
    #[error("failed to parse opinion market page {page}")]
    JsonFailed { source: reqwest::Error, page: u32 },
    #[error("failed to increment opinion page {page}")]
    PageCheckedAddFailed { page: u32 },
}

#[derive(Error, Debug)]
pub enum ListRelatedMarketsCommandOpinionMarketPageUrlError {
    #[error("failed to parse opinion markets URL")]
    ParseFailed { source: UrlParseError },
}

#[derive(Error, Debug)]
pub enum ListRelatedMarketsCommandWriteRelationLineError {
    #[error("failed to serialize relation to JSON")]
    SerializeFailed { source: serde_json::Error },
    #[error("failed to write short relation line")]
    WriteLineFailed { source: io::Error },
    #[error("failed to write relation newline")]
    WriteNewlineFailed { source: io::Error },
}
