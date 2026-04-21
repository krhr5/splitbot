pub mod bitcoin;
pub mod evm;
pub mod monero;
pub mod solana;

use crate::vault::ChainFamily;
use splitnow::NetworkId;

pub fn chain_family_for_network(network: NetworkId) -> Option<ChainFamily> {
    match network {
        NetworkId::Solana => Some(ChainFamily::Solana),
        NetworkId::Ethereum
        | NetworkId::Base
        | NetworkId::BinanceSmartChain
        | NetworkId::AvalancheCChain
        | NetworkId::Polygon
        | NetworkId::Cronos
        | NetworkId::ArbitrumOne
        | NetworkId::Optimism
        | NetworkId::EthereumClassic
        | NetworkId::AvalancheXChain
        | NetworkId::Plasma
        | NetworkId::Abstract
        | NetworkId::Hyperevm
        | NetworkId::Ink
        | NetworkId::XLayer
        | NetworkId::Story
        | NetworkId::Monad => Some(ChainFamily::Evm),
        NetworkId::Bitcoin | NetworkId::BitcoinLn => Some(ChainFamily::Bitcoin),
        NetworkId::Monero => Some(ChainFamily::Monero),
        _ => None,
    }
}

pub fn generate(
    chain_family: ChainFamily,
    label: impl Into<String>,
) -> anyhow::Result<crate::vault::StoredWallet> {
    match chain_family {
        ChainFamily::Solana => Ok(solana::generate(label)),
        ChainFamily::Evm => evm::generate(label),
        ChainFamily::Bitcoin => bitcoin::generate(label),
        ChainFamily::Monero => monero::generate(label),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_splitnow_networks_to_wallet_families() {
        assert_eq!(
            chain_family_for_network(NetworkId::Solana),
            Some(ChainFamily::Solana)
        );
        assert_eq!(
            chain_family_for_network(NetworkId::Base),
            Some(ChainFamily::Evm)
        );
        assert_eq!(
            chain_family_for_network(NetworkId::Bitcoin),
            Some(ChainFamily::Bitcoin)
        );
        assert_eq!(
            chain_family_for_network(NetworkId::Monero),
            Some(ChainFamily::Monero)
        );
        assert_eq!(chain_family_for_network(NetworkId::Ton), None);
    }
}
