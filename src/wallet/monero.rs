use crate::vault::{ChainFamily, StoredWallet, WalletSecret};
use anyhow::{Context, Result};
use monero::Network;
use monero::util::address::Address;
use monero::util::key::{KeyPair, PrivateKey};
use rand_core::{OsRng, RngCore};

pub fn generate(label: impl Into<String>) -> Result<StoredWallet> {
    let spend = random_private_key();
    let view = random_private_key();
    wallet_from_keys(label, spend, view)
}

pub fn import_keys_hex(
    label: impl Into<String>,
    private_spend_key_hex: &str,
    private_view_key_hex: &str,
) -> Result<StoredWallet> {
    let spend = parse_private_key_hex(private_spend_key_hex).context("parse Monero spend key")?;
    let view = parse_private_key_hex(private_view_key_hex).context("parse Monero view key")?;
    wallet_from_keys(label, spend, view)
}

pub fn private_keys_hex(wallet: &StoredWallet) -> Result<(String, String)> {
    let WalletSecret::Monero {
        private_spend_key_hex,
        private_view_key_hex,
    } = &wallet.secret
    else {
        anyhow::bail!("not a Monero wallet");
    };
    Ok((private_spend_key_hex.clone(), private_view_key_hex.clone()))
}

fn wallet_from_keys(
    label: impl Into<String>,
    spend: PrivateKey,
    view: PrivateKey,
) -> Result<StoredWallet> {
    let keys = KeyPair { spend, view };
    let address = Address::from_keypair(Network::Mainnet, &keys).to_string();
    Ok(StoredWallet {
        label: label.into(),
        chain_family: ChainFamily::Monero,
        address,
        secret: WalletSecret::Monero {
            private_spend_key_hex: hex::encode(keys.spend.to_bytes()),
            private_view_key_hex: hex::encode(keys.view.to_bytes()),
        },
    })
}

fn random_private_key() -> PrivateKey {
    loop {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        if let Ok(key) = PrivateKey::try_from(bytes) {
            return key;
        }
    }
}

fn parse_private_key_hex(value: &str) -> Result<PrivateKey> {
    let cleaned = value.trim().strip_prefix("0x").unwrap_or(value.trim());
    let bytes = hex::decode(cleaned).context("decode Monero private key hex")?;
    PrivateKey::from_slice(&bytes).context("decode Monero private key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_keys_roundtrip() {
        let wallet = generate("xmr").unwrap();
        let (spend, view) = private_keys_hex(&wallet).unwrap();
        let imported = import_keys_hex("imported", &spend, &view).unwrap();
        assert_eq!(wallet.address, imported.address);
        assert!(wallet.address.starts_with('4'));
    }
}
