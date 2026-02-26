use crate::{OrderType, Side, TokenId};
use alloy::signers::local::MnemonicBuilder;
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use errgonomic::handle;
use polymarket_client_sdk::auth::state::Authenticated as PolymarketAuthenticated;
use polymarket_client_sdk::auth::{Kind as PolymarketAuthKind, Signer};
use polymarket_client_sdk::clob::types::response::PostOrderResponse;
use polymarket_client_sdk::clob::types::{Order as PolymarketClobOrder, OrderStatusType, OrderType as PolymarketClobOrderType, Side as PolymarketClobSide, SignableOrder as PolymarketClobSignableOrder, SignatureType as PolymarketClobSignatureType};
use polymarket_client_sdk::clob::{Client as PolymarketClobClient, Config as PolymarketClobConfig};
use polymarket_client_sdk::error::Error as PolymarketError;
use polymarket_client_sdk::types::{Address, B256, ChainId, Decimal, U256};
use rust_decimal::prelude::ToPrimitive as _;
use std::io::{Write, stdout};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(clap::Parser, Clone, Debug)]
pub struct ClobPlaceLimitOrderCommand {
    #[arg(long, default_value = "https://clob.polymarket.com")]
    pub host: String,

    #[arg(long, default_value_t = polymarket_client_sdk::POLYGON)]
    pub chain_id: ChainId,

    /// Seed phrase (mnemonic words). Can also be provided via POLYMARKET_SEED_PHRASE env var.
    #[arg(long, env = "POLYMARKET_SEED_PHRASE")]
    pub seed_phrase: String,

    /// Optional seed phrase passphrase (BIP-39). Can also be provided via POLYMARKET_SEED_PHRASE_PASSWORD env var.
    #[arg(long, env = "POLYMARKET_SEED_PHRASE_PASSWORD")]
    pub seed_phrase_password: Option<String>,

    /// Account index used in derivation path m/44'/60'/0'/0/{index}
    #[arg(long, default_value_t = 0)]
    pub account_index: u32,

    #[arg(long, value_enum, default_value_t = ClobPlaceLimitOrderCommandSignatureType::Eoa)]
    pub signature_type: ClobPlaceLimitOrderCommandSignatureType,

    #[arg(long)]
    pub token_id: TokenId,

    #[arg(long, value_enum)]
    pub side: Side,

    #[arg(long)]
    pub price: Decimal,

    #[arg(long)]
    pub size: Decimal,

    #[arg(long, default_value_t = 0)]
    pub nonce: u64,

    #[arg(long, default_value = "1970-01-01T00:00:00Z")]
    pub expiration: DateTime<Utc>,

    #[arg(long = "type", value_enum, default_value_t = OrderType::Gtc)]
    pub order_type: OrderType,

    #[arg(long, default_value_t = false)]
    pub post_only: bool,

    #[arg(long)]
    pub funder: Option<Address>,

    #[arg(long, default_value_t = Address::ZERO)]
    pub taker: Address,
}

impl ClobPlaceLimitOrderCommand {
    pub async fn run(self) -> Result<ExitCode, ClobPlaceLimitOrderCommandRunError> {
        use ClobPlaceLimitOrderCommandRunError::*;
        let Self {
            host,
            chain_id,
            signature_type,
            funder,
            seed_phrase,
            seed_phrase_password,
            account_index,
            token_id,
            side,
            price,
            size,
            nonce,
            expiration,
            taker,
            order_type,
            post_only,
        } = self;
        let mnemonic_builder = MnemonicBuilder::english().phrase(seed_phrase);
        let mnemonic_builder = handle!(mnemonic_builder.index(account_index), MnemonicBuilderIndexFailed, account_index);
        let mnemonic_builder = match seed_phrase_password {
            Some(seed_phrase_password) => mnemonic_builder.password(seed_phrase_password),
            None => mnemonic_builder,
        };
        let signer = handle!(mnemonic_builder.build(), MnemonicBuilderBuildFailed, account_index).with_chain_id(Some(chain_id));
        let signature_type = PolymarketClobSignatureType::from(signature_type);
        let client_unauthenticated = handle!(PolymarketClobClient::new(&host, PolymarketClobConfig::default()), ClientNewFailed, host);
        let authentication_builder = client_unauthenticated
            .authentication_builder(&signer)
            .signature_type(signature_type);
        let authentication_builder = match funder {
            Some(funder) => authentication_builder.funder(funder),
            None => authentication_builder,
        };
        let client = handle!(authentication_builder.authenticate().await, AuthenticateFailed);
        let limit_order_builder = client
            .limit_order()
            .token_id(token_id)
            .side(side.into())
            .price(price)
            .size(size)
            .nonce(nonce)
            .expiration(expiration)
            .taker(taker)
            .order_type(order_type.into())
            .post_only(post_only);
        let signable_order = handle!(limit_order_builder.build().await, BuildLimitOrderFailed);
        let signed_order = handle!(client.sign(&signer, signable_order).await, SignOrderFailed);
        let response = handle!(client.post_order(signed_order).await, PostOrderFailed);

        let output = ClobPlaceLimitOrderCommandOutput::from(response);
        let mut stdout = stdout().lock();
        handle!(serde_json::to_writer(&mut stdout, &output), SerializeOutputFailed);
        handle!(stdout.write_all(b"\n"), WriteOutputNewlineFailed);
        Ok(ExitCode::SUCCESS)
    }
}

const LIMIT_ORDER_BUILD_USDC_DECIMALS: u32 = 6;
const LIMIT_ORDER_BUILD_LOT_SIZE_SCALE: u32 = 2;
const IEEE_754_INT_MAX: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug)]
pub struct BuildOverrideInput {
    pub signer: Address,
    pub signature_type: PolymarketClobSignatureType,
    pub token_id: TokenId,
    pub side: PolymarketClobSide,
    pub price: Decimal,
    pub size: Decimal,
    pub nonce: Option<u64>,
    pub expiration: Option<DateTime<Utc>>,
    pub taker: Option<Address>,
    pub order_type: Option<PolymarketClobOrderType>,
    pub post_only: Option<bool>,
    pub funder: Option<Address>,
}

pub async fn build_override<K: PolymarketAuthKind>(client: &PolymarketClobClient<PolymarketAuthenticated<K>>, input: BuildOverrideInput) -> Result<PolymarketClobSignableOrder, PolymarketError> {
    let BuildOverrideInput {
        signer,
        signature_type,
        token_id,
        side,
        price,
        size,
        nonce,
        expiration,
        taker,
        order_type,
        post_only,
        funder,
    } = input;
    if price.is_sign_negative() {
        return Err(PolymarketError::validation(format!("Unable to build Order due to negative price {price}")));
    }

    let fee_rate = client.fee_rate_bps(token_id).await?;
    let minimum_tick_size = client
        .tick_size(token_id)
        .await?
        .minimum_tick_size
        .as_decimal();
    let decimals = minimum_tick_size.scale();

    if price.scale() > minimum_tick_size.scale() {
        return Err(PolymarketError::validation(format!(
            "Unable to build Order: Price {price} has {} decimal places. Minimum tick size \
                {minimum_tick_size} has {} decimal places. Price decimal places <= minimum tick size decimal places",
            price.scale(),
            minimum_tick_size.scale()
        )));
    }

    let max_price = Decimal::ONE
        .checked_sub(minimum_tick_size)
        .ok_or_else(|| PolymarketError::validation(format!("Unable to subtract minimum tick size {minimum_tick_size} from one")))?;

    if price < minimum_tick_size || price > max_price {
        return Err(PolymarketError::validation(format!("Price {price} is too small or too large for the minimum tick size {minimum_tick_size}")));
    }

    if size.scale() > LIMIT_ORDER_BUILD_LOT_SIZE_SCALE {
        return Err(PolymarketError::validation(format!("Unable to build Order: Size {size} has {} decimal places. Maximum lot size is {LIMIT_ORDER_BUILD_LOT_SIZE_SCALE}", size.scale())));
    }

    if size.is_sign_negative() {
        return Err(PolymarketError::validation(format!("Unable to build Order due to negative size {size}")));
    }

    let nonce = nonce.unwrap_or(0);
    let expiration = expiration.unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let taker = taker.unwrap_or(Address::ZERO);
    let order_type = order_type.unwrap_or(PolymarketClobOrderType::GTC);
    let post_only = Some(post_only.unwrap_or(false));

    if !matches!(order_type, PolymarketClobOrderType::GTD) && expiration > DateTime::<Utc>::UNIX_EPOCH {
        return Err(PolymarketError::validation("Only GTD orders may have a non-zero expiration"));
    }

    if post_only == Some(true) && !matches!(order_type, PolymarketClobOrderType::GTC | PolymarketClobOrderType::GTD) {
        return Err(PolymarketError::validation("postOnly is only supported for GTC and GTD orders"));
    }

    let amount_scale = decimals
        .checked_add(LIMIT_ORDER_BUILD_LOT_SIZE_SCALE)
        .ok_or_else(|| PolymarketError::validation(format!("Unable to add decimal scales {decimals} and {LIMIT_ORDER_BUILD_LOT_SIZE_SCALE}")))?;
    let notional = size
        .checked_mul(price)
        .ok_or_else(|| PolymarketError::validation(format!("Unable to multiply size {size} and price {price}")))?;
    let notional = notional.trunc_with_scale(amount_scale);
    let (taker_amount, maker_amount) = match side {
        PolymarketClobSide::Buy => (size, notional),
        PolymarketClobSide::Sell => (notional, size),
        side => return Err(PolymarketError::validation(format!("Invalid side: {side}"))),
    };

    let salt = to_ieee_754_int_override(generate_seed_override()?);
    let expiration_u64 = expiration
        .timestamp()
        .to_u64()
        .ok_or_else(|| PolymarketError::validation(format!("Unable to represent expiration {expiration} as a u64")))?;

    let mut order = PolymarketClobOrder::default();
    order.salt = U256::from(salt);
    order.maker = funder.unwrap_or(signer);
    order.taker = taker;
    order.tokenId = token_id;
    order.makerAmount = U256::from(to_fixed_u128_override(maker_amount)?);
    order.takerAmount = U256::from(to_fixed_u128_override(taker_amount)?);
    order.side = side as u8;
    order.feeRateBps = U256::from(fee_rate.base_fee);
    order.nonce = U256::from(nonce);
    order.signer = signer;
    order.expiration = U256::from(expiration_u64);
    order.signatureType = signature_type as u8;

    let mut signable_order = PolymarketClobSignableOrder::default();
    signable_order.order = order;
    signable_order.order_type = order_type;
    signable_order.post_only = post_only;
    Ok(signable_order)
}

fn to_fixed_u128_override(amount: Decimal) -> Result<u128, PolymarketError> {
    let normalized = amount
        .normalize()
        .trunc_with_scale(LIMIT_ORDER_BUILD_USDC_DECIMALS);
    normalized
        .mantissa()
        .to_u128()
        .ok_or_else(|| PolymarketError::validation(format!("Unable to represent amount {normalized} as u128")))
}

fn generate_seed_override() -> Result<u64, PolymarketError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|source| PolymarketError::validation(format!("Unable to generate order seed from system time: {source}")))?;
    Ok(duration.as_secs() ^ u64::from(duration.subsec_nanos()))
}

fn to_ieee_754_int_override(salt: u64) -> u64 {
    salt & IEEE_754_INT_MAX
}

#[derive(Error, Debug)]
pub enum ClobPlaceLimitOrderCommandRunError {
    #[error("failed to set mnemonic derivation index '{account_index}'")]
    MnemonicBuilderIndexFailed { source: alloy::signers::local::LocalSignerError, account_index: u32 },
    #[error("failed to build signer from mnemonic at account index '{account_index}'")]
    MnemonicBuilderBuildFailed { source: alloy::signers::local::LocalSignerError, account_index: u32 },
    #[error("failed to initialize clob client for host '{host}'")]
    ClientNewFailed { source: polymarket_client_sdk::error::Error, host: String },
    #[error("failed to authenticate clob client")]
    AuthenticateFailed { source: polymarket_client_sdk::error::Error },
    #[error("failed to build limit order")]
    BuildLimitOrderFailed { source: polymarket_client_sdk::error::Error },
    #[error("failed to sign order")]
    SignOrderFailed { source: polymarket_client_sdk::error::Error },
    #[error("failed to post order")]
    PostOrderFailed { source: polymarket_client_sdk::error::Error },
    #[error("failed to serialize output")]
    SerializeOutputFailed { source: serde_json::Error },
    #[error("failed to write output newline")]
    WriteOutputNewlineFailed { source: std::io::Error },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum ClobPlaceLimitOrderCommandSignatureType {
    Eoa,
    Proxy,
    GnosisSafe,
}

impl From<ClobPlaceLimitOrderCommandSignatureType> for PolymarketClobSignatureType {
    fn from(input: ClobPlaceLimitOrderCommandSignatureType) -> Self {
        use ClobPlaceLimitOrderCommandSignatureType::*;
        match input {
            Eoa => Self::Eoa,
            Proxy => Self::Proxy,
            GnosisSafe => Self::GnosisSafe,
        }
    }
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClobPlaceLimitOrderCommandOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_msg: Option<String>,
    pub making_amount: Decimal,
    pub taking_amount: Decimal,
    #[serde(rename = "orderID")]
    pub order_id: String,
    pub status: OrderStatusType,
    pub success: bool,
    pub transaction_hashes: Vec<B256>,
    pub trade_ids: Vec<String>,
}

impl From<PostOrderResponse> for ClobPlaceLimitOrderCommandOutput {
    fn from(input: PostOrderResponse) -> Self {
        let PostOrderResponse {
            error_msg,
            making_amount,
            taking_amount,
            order_id,
            status,
            success,
            transaction_hashes,
            trade_ids,
            ..
        } = input;
        Self {
            error_msg,
            making_amount,
            taking_amount,
            order_id,
            status,
            success,
            transaction_hashes,
            trade_ids,
        }
    }
}
