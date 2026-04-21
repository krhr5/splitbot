use crate::vault::{ChainFamily, StoredWallet, WalletSecret};
use anyhow::{Context, Result};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, CompressedPublicKey, Network, PrivateKey};

pub fn generate(label: impl Into<String>) -> Result<StoredWallet> {
    let private_key = PrivateKey::generate(Network::Bitcoin);
    wallet_from_private_key(label, private_key)
}

pub fn import_wif(label: impl Into<String>, wif: &str) -> Result<StoredWallet> {
    let private_key = PrivateKey::from_wif(wif.trim()).context("parse Bitcoin WIF")?;
    wallet_from_private_key(label, private_key)
}

pub fn wif(wallet: &StoredWallet) -> Result<String> {
    let WalletSecret::Bitcoin { wif } = &wallet.secret else {
        anyhow::bail!("not a Bitcoin wallet");
    };
    Ok(wif.clone())
}

fn wallet_from_private_key(
    label: impl Into<String>,
    private_key: PrivateKey,
) -> Result<StoredWallet> {
    let secp = Secp256k1::new();
    let public_key = CompressedPublicKey::from_private_key(&secp, &private_key)
        .context("derive compressed Bitcoin public key")?;
    let address = Address::p2wpkh(&public_key, Network::Bitcoin).to_string();
    Ok(StoredWallet {
        label: label.into(),
        chain_family: ChainFamily::Bitcoin,
        address,
        secret: WalletSecret::Bitcoin {
            wif: private_key.to_wif(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_wif_roundtrips() {
        let wallet = generate("btc").unwrap();
        let secret = wif(&wallet).unwrap();
        let imported = import_wif("imported", &secret).unwrap();
        assert_eq!(wallet.address, imported.address);
        assert!(wallet.address.starts_with("bc1q"));
    }
}
