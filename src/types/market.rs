use crate::{Amount, ConditionId, ConvertVecTokenRawToTokensError, NegRisk, QuestionId, Rewards, TokenId, Tokens, TryFromNegRiskTripleError, from_chrono_date_time, into_chrono_date_time};
use alloy::primitives::Address;
use derive_more::{From, Into};
use errgonomic::handle;
use polymarket_client_sdk::clob::types::response::{MarketResponse, Rewards as RewardsRaw, Token as TokenRaw};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Duration, OffsetDateTime};

#[derive(From, Into, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Market {
    pub question: String,
    pub description: String,
    pub market_slug: String,
    pub icon: String,
    pub image: String,
    /// Optional condition id provided by the API.
    pub condition_id: Option<ConditionId>,
    /// Optional question id provided by the API.
    pub question_id: Option<QuestionId>,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub enable_order_book: bool,
    pub accepting_orders: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepting_order_timestamp: Option<OffsetDateTime>,
    pub minimum_order_size: Amount,
    pub minimum_tick_size: Amount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date_iso: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_start_time: Option<OffsetDateTime>,
    pub seconds_delay: Duration,
    pub fpmm: Option<Address>,
    pub maker_base_fee: Amount,
    pub taker_base_fee: Amount,
    pub rewards: Rewards,
    pub tokens: Tokens,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neg_risk: Option<NegRisk>,
    pub is_50_50_outcome: bool,
    pub notifications_enabled: bool,
    pub tags: Vec<String>,
}

impl Market {
    pub fn is_tradeable(&self) -> bool {
        self.active && !self.closed && !self.archived && self.accepting_orders && self.enable_order_book
    }

    pub fn token_ids_tuple(&self) -> (TokenId, TokenId) {
        self.tokens.token_ids_tuple()
    }

    pub fn token_ids_array(&self) -> [TokenId; 2] {
        self.tokens.token_ids_array()
    }
}

/// NOTE: Some markets have an invalid `neg_risk_market_id` (e.g. "0x12309") because they were created by Polymarket just for testing
impl TryFrom<MarketResponse> for Market {
    type Error = ConvertMarketResponseToMarketError;

    fn try_from(market: MarketResponse) -> Result<Self, Self::Error> {
        use ConvertMarketResponseToMarketError::*;
        let MarketResponse {
            enable_order_book,
            active,
            closed,
            archived,
            accepting_orders,
            accepting_order_timestamp,
            minimum_order_size,
            minimum_tick_size,
            condition_id,
            question_id,
            question,
            description,
            market_slug,
            end_date_iso,
            game_start_time,
            seconds_delay,
            fpmm,
            maker_base_fee,
            taker_base_fee,
            notifications_enabled,
            neg_risk,
            neg_risk_market_id,
            neg_risk_request_id,
            icon,
            image,
            rewards,
            is_50_50_outcome,
            tokens,
            tags,
            ..
        } = market;
        let rewards = rewards.into();
        let accepting_order_timestamp = handle!(
            accepting_order_timestamp
                .map(from_chrono_date_time)
                .transpose(),
            AcceptingOrderTimestampFromChronoDateTimeFailed
        );
        let end_date_iso = handle!(end_date_iso.map(from_chrono_date_time).transpose(), EndDateIsoFromChronoDateTimeFailed);
        let game_start_time = handle!(game_start_time.map(from_chrono_date_time).transpose(), GameStartTimeFromChronoDateTimeFailed);
        let seconds_delay = handle!(i64::try_from(seconds_delay), SecondsDelayTryFromFailed, seconds_delay);
        let seconds_delay = Duration::seconds(seconds_delay);
        let neg_risk = handle!(NegRisk::try_from_neg_risk_triple(neg_risk, neg_risk_market_id, neg_risk_request_id), NegRiskTryFromTripleFailed);
        let tokens = handle!(Tokens::try_from(tokens), TokensTryFromFailed);
        Ok(Self {
            question,
            description,
            market_slug,
            icon,
            image,
            condition_id,
            question_id,
            active,
            closed,
            archived,
            enable_order_book,
            accepting_orders,
            accepting_order_timestamp,
            minimum_order_size,
            minimum_tick_size,
            end_date_iso,
            game_start_time,
            seconds_delay,
            fpmm,
            maker_base_fee,
            taker_base_fee,
            rewards,
            tokens,
            neg_risk,
            is_50_50_outcome,
            notifications_enabled,
            tags,
        })
    }
}

#[derive(Error, Debug)]
pub enum ConvertMarketResponseToMarketError {
    #[error("failed to convert accepting_order_timestamp")]
    AcceptingOrderTimestampFromChronoDateTimeFailed { source: time::error::ComponentRange },
    #[error("failed to convert end_date_iso")]
    EndDateIsoFromChronoDateTimeFailed { source: time::error::ComponentRange },
    #[error("failed to convert game_start_time")]
    GameStartTimeFromChronoDateTimeFailed { source: time::error::ComponentRange },
    #[error("failed to convert seconds_delay '{seconds_delay}'")]
    SecondsDelayTryFromFailed { source: core::num::TryFromIntError, seconds_delay: u64 },
    #[error("failed to convert neg_risk fields")]
    NegRiskTryFromTripleFailed { source: TryFromNegRiskTripleError },
    #[error("failed to convert tokens")]
    TokensTryFromFailed { source: ConvertVecTokenRawToTokensError },
}

impl From<Market> for MarketResponse {
    fn from(market: Market) -> Self {
        let Market {
            question,
            description,
            market_slug,
            icon,
            image,
            condition_id,
            question_id,
            active,
            closed,
            archived,
            enable_order_book,
            accepting_orders,
            accepting_order_timestamp,
            minimum_order_size,
            minimum_tick_size,
            end_date_iso,
            game_start_time,
            seconds_delay,
            fpmm,
            maker_base_fee,
            taker_base_fee,
            rewards,
            tokens,
            neg_risk,
            is_50_50_outcome,
            notifications_enabled,
            tags,
        } = market;
        let accepting_order_timestamp = accepting_order_timestamp.map(|timestamp| into_chrono_date_time(timestamp).expect("accepting_order_timestamp should convert because it originated from TryFrom"));
        let end_date_iso = end_date_iso.map(|timestamp| into_chrono_date_time(timestamp).expect("end_date_iso should convert because it originated from TryFrom"));
        let game_start_time = game_start_time.map(|timestamp| into_chrono_date_time(timestamp).expect("game_start_time should convert because it originated from TryFrom"));
        let seconds_delay = seconds_delay.whole_seconds();
        let seconds_delay = u64::try_from(seconds_delay).map_or(0, |value| value);
        let (neg_risk, neg_risk_market_id, neg_risk_request_id) = neg_risk
            .map(Into::into)
            .unwrap_or_else(|| (false, None, None));
        let rewards: RewardsRaw = rewards.into();
        let tokens: Vec<TokenRaw> = tokens.into();
        MarketResponse::builder()
            .enable_order_book(enable_order_book)
            .active(active)
            .closed(closed)
            .archived(archived)
            .accepting_orders(accepting_orders)
            .maybe_accepting_order_timestamp(accepting_order_timestamp)
            .minimum_order_size(minimum_order_size)
            .minimum_tick_size(minimum_tick_size)
            .maybe_condition_id(condition_id)
            .maybe_question_id(question_id)
            .question(question)
            .description(description)
            .market_slug(market_slug)
            .maybe_end_date_iso(end_date_iso)
            .maybe_game_start_time(game_start_time)
            .seconds_delay(seconds_delay)
            .maybe_fpmm(fpmm)
            .maker_base_fee(maker_base_fee)
            .taker_base_fee(taker_base_fee)
            .notifications_enabled(notifications_enabled)
            .neg_risk(neg_risk)
            .maybe_neg_risk_market_id(neg_risk_market_id)
            .maybe_neg_risk_request_id(neg_risk_request_id)
            .icon(icon)
            .image(image)
            .rewards(rewards)
            .is_50_50_outcome(is_50_50_outcome)
            .tokens(tokens)
            .tags(tags)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NEXT_CURSOR_STOP, REFRESH_TEST_CACHE_ENV, assert_round_trip_own, parse_boolish_env_var, progress_report_line, to_tmp_path};
    use async_jsonl::{Jsonl, JsonlDeserialize};
    use async_stream::stream;
    use futures::{Stream, StreamExt};
    use polymarket_client_sdk::clob::Client;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;
    use tokio::fs::{self, File};
    use tokio::io::AsyncWriteExt;

    #[test]
    fn must_round_trip_fixture() {
        let input = include_str!("../../fixtures/market.json");
        let market_response: MarketResponse = serde_json::de::from_str(input).unwrap();
        let market = Market::try_from(market_response.clone()).unwrap();
        assert_eq!(market.question, "Will Donald Trump win the 2024 US Presidential Election?");
        let market_response_round_trip = MarketResponse::from(market);
        assert_eq!(market_response_round_trip, market_response);
    }

    #[tokio::test]
    async fn must_round_trip_data() -> ExitCode {
        let inputs = get_market_response_stream();
        assert_round_trip_own::<MarketResponse, Market, <Market as TryFrom<MarketResponse>>::Error>(inputs).await
    }

    fn get_market_response_stream() -> impl Stream<Item = MarketResponse> {
        let cache_path = market_response_cache_path();
        let stream = stream! {
            use GetMarketResponseStreamError::*;
            let refresh_requested = match parse_boolish_env_var(REFRESH_TEST_CACHE_ENV) {
                Ok(Some(value)) => value,
                Ok(None) => false,
                Err(source) => {
                    yield Err(ParseRefreshEnvVarFailed {
                        source,
                        var: REFRESH_TEST_CACHE_ENV.to_string(),
                    });
                    return;
                }
            };
            if refresh_requested {
                let stream = refresh_market_response_cache(&cache_path);
                futures::pin_mut!(stream);
                while let Some(market) = stream.next().await {
                    yield Ok(market);
                }
            } else if cache_path.exists() {
                let reader = match Jsonl::from_path(&cache_path).await {
                    Ok(reader) => reader,
                    Err(_source) => {
                        yield Err(OpenCacheFailed {
                            cache_path: cache_path.clone(),
                        });
                        return;
                    }
                };
                let stream = reader.deserialize::<MarketResponse>();
                futures::pin_mut!(stream);
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(market) => yield Ok(market),
                        Err(_source) => {
                            yield Err(ParseCacheFailed {
                                cache_path: cache_path.clone(),
                            });
                            return;
                        }
                    }
                }
            } else {
                yield Err(CacheMissing {
                    cache_path: cache_path.clone(),
                });
            }
        };
        stream.map(|result| match result {
            Ok(market) => market,
            Err(error) => panic!("{error}"),
        })
    }

    const DEFAULT_MARKET_RESPONSE_CACHE_PATH: &str = "cache.local/market_response.all.jsonl";

    fn market_response_cache_path() -> PathBuf {
        PathBuf::from(DEFAULT_MARKET_RESPONSE_CACHE_PATH)
    }

    fn refresh_market_response_cache(cache_path: &Path) -> impl Stream<Item = MarketResponse> {
        let cache_path = cache_path.to_path_buf();
        let stream = stream! {
            use RefreshMarketResponseCacheError::*;
            let cache_dir = cache_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
            if let Err(source) = fs::create_dir_all(&cache_dir).await {
                yield Err(CreateCacheDirFailed {
                    source,
                    cache_dir,
                });
                return;
            }
            let temp_path = to_tmp_path(&cache_path);
            match fs::remove_file(&temp_path).await {
                Ok(()) => (),
                Err(source) if source.kind() == io::ErrorKind::NotFound => (),
                Err(source) => {
                    yield Err(RemoveTempFileFailed {
                        source,
                        temp_path,
                    });
                    return;
                }
            }
            let mut file = match File::create(&temp_path).await {
                Ok(file) => file,
                Err(source) => {
                    yield Err(CreateTempFileFailed {
                        source,
                        temp_path,
                    });
                    return;
                }
            };
            let client = Client::default();
            let mut downloaded: u64 = 0;
            let mut next_cursor: Option<String> = None;

            loop {
                let page = match client.markets(next_cursor.clone()).await {
                    Ok(page) => page,
                    Err(source) => {
                        yield Err(FetchMarketsFailed {
                            source,
                            next_cursor,
                        });
                        return;
                    }
                };
                let page_next_cursor = page.next_cursor.clone();
                for market in page.data {
                    let line = match serde_json::to_string(&market) {
                        Ok(line) => line,
                        Err(source) => {
                            yield Err(SerializeMarketFailed {
                                source,
                                market: Box::new(market),
                            });
                            return;
                        }
                    };
                    if let Err(source) = file.write_all(line.as_bytes()).await {
                        yield Err(WriteTempFileFailed {
                            source,
                            temp_path,
                        });
                        return;
                    }
                    if let Err(source) = file.write_all(b"\n").await {
                        yield Err(WriteTempFileFailed {
                            source,
                            temp_path,
                        });
                        return;
                    }
                    downloaded = downloaded.saturating_add(1);
                    if downloaded.is_multiple_of(100) {
                        eprintln!("{}", progress_report_line("Downloading objects", downloaded, None));
                    }
                    yield Ok(market);
                }
                if page_next_cursor == NEXT_CURSOR_STOP {
                    break;
                }
                next_cursor = Some(page_next_cursor);
            }
            if !downloaded.is_multiple_of(100) {
                eprintln!("{}", progress_report_line("Downloading objects", downloaded, None));
            }
            if let Err(source) = file.flush().await {
                yield Err(FlushTempFileFailed {
                    source,
                    temp_path,
                });
                return;
            }
            if let Err(source) = file.sync_all().await {
                yield Err(SyncTempFileFailed {
                    source,
                    temp_path,
                });
                return;
            }
            drop(file);
            if let Err(source) = fs::rename(&temp_path, &cache_path).await {
                yield Err(PersistTempFileFailed {
                    source,
                    temp_path,
                    cache_path,
                });
                return;
            }
        };
        stream.map(|result| match result {
            Ok(market) => market,
            Err(error) => panic!("{error}"),
        })
    }

    #[allow(clippy::enum_variant_names)]
    #[derive(Error, Debug)]
    pub enum RefreshMarketResponseCacheError {
        #[error("failed to create cache directory '{cache_dir}'")]
        CreateCacheDirFailed { source: io::Error, cache_dir: PathBuf },
        #[error("failed to remove temp cache file '{temp_path}'")]
        RemoveTempFileFailed { source: io::Error, temp_path: PathBuf },
        #[error("failed to create temp cache file '{temp_path}'")]
        CreateTempFileFailed { source: io::Error, temp_path: PathBuf },
        #[error("failed to fetch markets page")]
        FetchMarketsFailed { source: polymarket_client_sdk::error::Error, next_cursor: Option<String> },
        #[error("failed to serialize market response")]
        SerializeMarketFailed { source: serde_json::Error, market: Box<MarketResponse> },
        #[error("failed to write temp cache file '{temp_path}'")]
        WriteTempFileFailed { source: io::Error, temp_path: PathBuf },
        #[error("failed to flush temp cache file '{temp_path}'")]
        FlushTempFileFailed { source: io::Error, temp_path: PathBuf },
        #[error("failed to sync temp cache file '{temp_path}'")]
        SyncTempFileFailed { source: io::Error, temp_path: PathBuf },
        #[error("failed to persist temp cache file '{temp_path}' to '{cache_path}'")]
        PersistTempFileFailed { source: io::Error, temp_path: PathBuf, cache_path: PathBuf },
    }

    #[allow(clippy::enum_variant_names)]
    #[derive(Error, Debug)]
    pub enum GetMarketResponseStreamError {
        #[error("failed to parse env var '{var}'")]
        ParseRefreshEnvVarFailed { source: crate::ParseBoolishEnvVarError, var: String },
        #[error("market response cache not found at '{cache_path}'")]
        CacheMissing { cache_path: PathBuf },
        #[error("failed to open market response cache at '{cache_path}'")]
        OpenCacheFailed { cache_path: PathBuf },
        #[error("failed to parse market response cache at '{cache_path}'")]
        ParseCacheFailed { cache_path: PathBuf },
    }
}
