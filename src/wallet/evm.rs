use crate::vault::{ChainFamily, StoredWallet, WalletSecret};
use alloy_primitives::Address;
use anyhow::{Context, Result};
use k256::ecdsa::SigningKey;
use rand_core::OsRng;

pub fn generate(label: impl Into<String>) -> Result<StoredWallet> {
    let signing_key = SigningKey::random(&mut OsRng);
    Ok(wallet_from_signing_key(label, signing_key))
}

pub fn import_private_key_hex(
    label: impl Into<String>,
    private_key_hex: &str,
) -> Result<StoredWallet> {
    let cleaned = private_key_hex
        .trim()
        .strip_prefix("0x")
        .unwrap_or(private_key_hex.trim());
    let bytes = hex::decode(cleaned).context("decode EVM private key hex")?;
    let signing_key = SigningKey::from_slice(&bytes).context("parse EVM private key")?;
    Ok(wallet_from_signing_key(label, signing_key))
}

pub fn private_key_hex(wallet: &StoredWallet) -> Result<String> {
    let WalletSecret::Evm { private_key_hex } = &wallet.secret else {
        anyhow::bail!("not an EVM wallet");
    };
    Ok(private_key_hex.clone())
}

fn wallet_from_signing_key(label: impl Into<String>, signing_key: SigningKey) -> StoredWallet {
    let address = Address::from_private_key(&signing_key).to_checksum(None);
    let private_key_hex = format!("0x{}", hex::encode(signing_key.to_bytes()));
    StoredWallet {
        label: label.into(),
        chain_family: ChainFamily::Evm,
        address,
        secret: WalletSecret::Evm { private_key_hex },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_roundtrips() {
        let wallet = generate("evm").unwrap();
        let secret = private_key_hex(&wallet).unwrap();
        let imported = import_private_key_hex("imported", &secret).unwrap();
        assert_eq!(wallet.address, imported.address);
        assert!(wallet.address.starts_with("0x"));
    }
}
