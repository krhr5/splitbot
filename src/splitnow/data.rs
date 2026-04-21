use crate::splitnow::client::AvailableExchanger;
use splitnow::{AssetId, ExchangerId, NetworkId};

pub const ASSETS: &[AssetId] = &[
    AssetId::Sol,
    AssetId::Usdc,
    AssetId::Usdt,
    AssetId::Btc,
    AssetId::Eth,
    AssetId::Bnb,
    AssetId::Avax,
    AssetId::Pol,
    AssetId::Ada,
    AssetId::Dot,
    AssetId::Doge,
    AssetId::Ltc,
    AssetId::Xrp,
    AssetId::Xlm,
    AssetId::Trx,
    AssetId::Atom,
    AssetId::Near,
    AssetId::Sui,
    AssetId::Apt,
    AssetId::Ton,
    AssetId::Xmr,
    AssetId::Bch,
    AssetId::Etc,
    AssetId::Fil,
    AssetId::Hbar,
    AssetId::Algo,
];

pub const NETWORKS: &[NetworkId] = &[
    NetworkId::Solana,
    NetworkId::Ethereum,
    NetworkId::Base,
    NetworkId::ArbitrumOne,
    NetworkId::Optimism,
    NetworkId::Polygon,
    NetworkId::BinanceSmartChain,
    NetworkId::AvalancheCChain,
    NetworkId::Bitcoin,
    NetworkId::BitcoinCash,
    NetworkId::Litecoin,
    NetworkId::Dogecoin,
    NetworkId::Ripple,
    NetworkId::Stellar,
    NetworkId::Tron,
    NetworkId::Cosmos,
    NetworkId::Near,
    NetworkId::Sui,
    NetworkId::Aptos,
    NetworkId::Ton,
    NetworkId::Monero,
    NetworkId::EthereumClassic,
    NetworkId::Filecoin,
    NetworkId::Hedera,
    NetworkId::Algorand,
    NetworkId::Cardano,
    NetworkId::Polkadot,
];

pub fn asset_label(a: AssetId) -> &'static str {
    match a {
        AssetId::Sol => "SOL",
        AssetId::Usdc => "USDC",
        AssetId::Usdt => "USDT",
        AssetId::Btc => "BTC",
        AssetId::Eth => "ETH",
        AssetId::Bnb => "BNB",
        AssetId::Avax => "AVAX",
        AssetId::Pol => "POL",
        AssetId::Ada => "ADA",
        AssetId::Dot => "DOT",
        AssetId::Doge => "DOGE",
        AssetId::Ltc => "LTC",
        AssetId::Xrp => "XRP",
        AssetId::Xlm => "XLM",
        AssetId::Trx => "TRX",
        AssetId::Atom => "ATOM",
        AssetId::Near => "NEAR",
        AssetId::Sui => "SUI",
        AssetId::Apt => "APT",
        AssetId::Ton => "TON",
        AssetId::Xmr => "XMR",
        AssetId::Bch => "BCH",
        AssetId::Etc => "ETC",
        AssetId::Fil => "FIL",
        AssetId::Hbar => "HBAR",
        AssetId::Algo => "ALGO",
        _ => "?",
    }
}

pub fn network_label(n: NetworkId) -> &'static str {
    match n {
        NetworkId::Solana => "Solana",
        NetworkId::Ethereum => "Ethereum",
        NetworkId::Base => "Base",
        NetworkId::ArbitrumOne => "Arbitrum One",
        NetworkId::Optimism => "Optimism",
        NetworkId::Polygon => "Polygon",
        NetworkId::BinanceSmartChain => "BSC",
        NetworkId::AvalancheCChain => "Avalanche C",
        NetworkId::Bitcoin => "Bitcoin",
        NetworkId::BitcoinCash => "Bitcoin Cash",
        NetworkId::Litecoin => "Litecoin",
        NetworkId::Dogecoin => "Dogecoin",
        NetworkId::Ripple => "Ripple",
        NetworkId::Stellar => "Stellar",
        NetworkId::Tron => "Tron",
        NetworkId::Cosmos => "Cosmos",
        NetworkId::Near => "Near",
        NetworkId::Sui => "Sui",
        NetworkId::Aptos => "Aptos",
        NetworkId::Ton => "Ton",
        NetworkId::Monero => "Monero",
        NetworkId::EthereumClassic => "Ethereum Classic",
        NetworkId::Filecoin => "Filecoin",
        NetworkId::Hedera => "Hedera",
        NetworkId::Algorand => "Algorand",
        NetworkId::Cardano => "Cardano",
        NetworkId::Polkadot => "Polkadot",
        _ => "?",
    }
}

pub fn exchanger_label(e: ExchangerId) -> String {
    format!("{e:?}")
}

pub fn fallback_exchangers() -> Vec<AvailableExchanger> {
    use ExchangerId::{
        Binance, Bybit, Changehero, Changelly, Changenow, Fixedfloat, Godex, Kucoin, Mexc,
        Sideshift, Simpleswap, Stealthex,
    };

    [
        Binance, Bybit, Kucoin, Mexc, Changelly, Changenow, Changehero, Fixedfloat, Sideshift,
        Simpleswap, Stealthex, Godex,
    ]
    .into_iter()
    .map(|id| AvailableExchanger {
        id,
        name: format!("{id:?}"),
    })
    .collect()
}
