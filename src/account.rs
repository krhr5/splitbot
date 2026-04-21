use crate::app::App;
use crate::events::AppEvent;
use crate::vault::{ChainFamily, StoredWallet};
use anyhow::{Context, Result};
use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

const SOLANA_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const SOL_PRICE_URL: &str =
    "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd";
const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;

#[derive(Debug, Clone)]
pub struct AccountBalances {
    pub total_sol: f64,
    pub total_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct RpcBalanceValue {
    value: u64,
}

#[derive(Debug, Deserialize)]
struct RpcBalanceResponse {
    result: RpcBalanceValue,
}

#[derive(Debug, Deserialize)]
struct PricePoint {
    usd: f64,
}

#[derive(Debug, Deserialize)]
struct PriceResponse {
    solana: PricePoint,
}

pub fn refresh_balances(app: &mut App) {
    let wallets = app
        .vault
        .as_ref()
        .map(|vault| vault.wallets().to_vec())
        .unwrap_or_default();
    app.account.balance_loading = true;
    app.account.balance_error = None;
    let tx = app.tx.clone();
    tokio::spawn(async move {
        let result = fetch_account_balances(&wallets)
            .await
            .map_err(|e| e.to_string());
        let _ = tx.send(AppEvent::AccountBalancesLoaded(result));
    });
}

async fn fetch_account_balances(wallets: &[StoredWallet]) -> Result<AccountBalances> {
    if wallets.is_empty() {
        return Ok(AccountBalances {
            total_sol: 0.0,
            total_usd: Some(0.0),
        });
    }

    let http = Client::new();
    let balance_futures = wallets
        .iter()
        .filter(|wallet| wallet.chain_family == ChainFamily::Solana)
        .map(|wallet| fetch_wallet_balance_lamports(&http, &wallet.address));
    let balance_results = join_all(balance_futures).await;

    let mut total_lamports = 0u64;
    for result in balance_results {
        let lamports = result?;
        total_lamports = total_lamports
            .checked_add(lamports)
            .context("wallet balance total overflow")?;
    }

    let total_sol = total_lamports as f64 / LAMPORTS_PER_SOL;
    let total_usd = fetch_sol_price_usd(&http)
        .await
        .ok()
        .map(|price| total_sol * price);

    Ok(AccountBalances {
        total_sol,
        total_usd,
    })
}

async fn fetch_wallet_balance_lamports(http: &Client, pubkey: &str) -> Result<u64> {
    let response = http
        .post(SOLANA_RPC_URL)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [pubkey],
        }))
        .send()
        .await
        .context("request Solana wallet balance")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .with_context(|| format!("read balance body for {pubkey}"))?;
    if !status.is_success() {
        anyhow::bail!("solana rpc http {status}");
    }

    let parsed: RpcBalanceResponse =
        serde_json::from_str(&body).context("parse Solana balance response")?;
    Ok(parsed.result.value)
}

async fn fetch_sol_price_usd(http: &Client) -> Result<f64> {
    let response = http
        .get(SOL_PRICE_URL)
        .send()
        .await
        .context("request SOL price")?;
    let status = response.status();
    let body = response.text().await.context("read SOL price body")?;
    if !status.is_success() {
        anyhow::bail!("SOL price http {status}");
    }

    let parsed: PriceResponse = serde_json::from_str(&body).context("parse SOL price response")?;
    Ok(parsed.solana.usd)
}
