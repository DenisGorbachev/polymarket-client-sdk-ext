use crate::{OrderType, Side, TokenId};
use alloy::signers::local::MnemonicBuilder;
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use errgonomic::handle;
use polymarket_client_sdk::auth::Signer;
use polymarket_client_sdk::clob::types::response::PostOrderResponse;
use polymarket_client_sdk::clob::types::{OrderStatusType, SignatureType as PolymarketClobSignatureType};
use polymarket_client_sdk::clob::{Client as PolymarketClobClient, Config as PolymarketClobConfig};
use polymarket_client_sdk::types::{Address, B256, ChainId, Decimal};
use std::io::{Write, stdout};
use std::process::ExitCode;
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

    #[arg(long)]
    pub nonce: Option<u64>,

    #[arg(long)]
    pub expiration: Option<DateTime<Utc>>,

    #[arg(long, value_enum)]
    pub r#type: Option<OrderType>,

    #[arg(long)]
    pub post_only: Option<bool>,

    #[arg(long)]
    pub funder: Option<Address>,

    #[arg(long)]
    pub taker: Option<Address>,
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
            r#type,
            post_only,
        } = self;
        let mnemonic_builder = MnemonicBuilder::english().phrase(seed_phrase);
        let mnemonic_builder = handle!(mnemonic_builder.index(account_index), MnemonicBuilderIndexFailed, account_index);
        let mnemonic_builder = match seed_phrase_password {
            Some(seed_phrase_password) => mnemonic_builder.password(seed_phrase_password),
            None => mnemonic_builder,
        };
        let signer = handle!(mnemonic_builder.build(), MnemonicBuilderBuildFailed, account_index).with_chain_id(Some(chain_id));
        let client_unauthenticated = handle!(PolymarketClobClient::new(&host, PolymarketClobConfig::default()), ClientNewFailed, host);
        let authentication_builder = client_unauthenticated
            .authentication_builder(&signer)
            .signature_type(signature_type.into());
        let authentication_builder = match funder {
            Some(funder) => authentication_builder.funder(funder),
            None => authentication_builder,
        };
        let client = handle!(authentication_builder.authenticate().await, AuthenticateFailed);

        let mut limit_order_builder = client
            .limit_order()
            .token_id(token_id)
            .side(side.into())
            .price(price)
            .size(size);
        limit_order_builder = match nonce {
            Some(nonce) => limit_order_builder.nonce(nonce),
            None => limit_order_builder,
        };
        limit_order_builder = match expiration {
            Some(expiration) => limit_order_builder.expiration(expiration),
            None => limit_order_builder,
        };
        limit_order_builder = match taker {
            Some(taker) => limit_order_builder.taker(taker),
            None => limit_order_builder,
        };
        limit_order_builder = match r#type {
            Some(order_type) => limit_order_builder.order_type(order_type.into()),
            None => limit_order_builder,
        };
        limit_order_builder = match post_only {
            Some(post_only) => limit_order_builder.post_only(post_only),
            None => limit_order_builder,
        };

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
