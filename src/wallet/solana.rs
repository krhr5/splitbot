use crate::vault::{ChainFamily, StoredWallet, WalletSecret};
use anyhow::{Context, Result, anyhow};
use solana_keypair::Keypair;
use solana_signer::Signer;

pub fn generate(label: impl Into<String>) -> StoredWallet {
    let kp = Keypair::new();
    let address = kp.pubkey().to_string();
    let secret_bytes = kp.to_bytes();
    StoredWallet {
        label: label.into(),
        chain_family: ChainFamily::Solana,
        address,
        secret: WalletSecret::Solana {
            bytes: secret_bytes,
        },
    }
}

pub fn import_base58(label: impl Into<String>, secret_b58: &str) -> Result<StoredWallet> {
    let bytes = bs58::decode(secret_b58.trim())
        .into_vec()
        .context("decode base58 secret")?;
    let arr: [u8; 64] = bytes
        .try_into()
        .map_err(|v: Vec<u8>| anyhow!("expected 64-byte secret, got {}", v.len()))?;
    let kp = Keypair::try_from(&arr[..]).map_err(|e| anyhow!("invalid keypair: {e}"))?;
    Ok(StoredWallet {
        label: label.into(),
        chain_family: ChainFamily::Solana,
        address: kp.pubkey().to_string(),
        secret: WalletSecret::Solana { bytes: arr },
    })
}

pub fn secret_base58(w: &StoredWallet) -> Result<String> {
    let WalletSecret::Solana { bytes } = &w.secret else {
        anyhow::bail!("not a Solana wallet");
    };
    Ok(bs58::encode(bytes).into_string())
}

#[allow(dead_code)]
pub fn keypair_from_stored(w: &StoredWallet) -> Result<Keypair> {
    let WalletSecret::Solana { bytes } = &w.secret else {
        anyhow::bail!("not a Solana wallet");
    };
    Keypair::try_from(&bytes[..]).map_err(|e| anyhow!("invalid stored keypair: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_rehydrate() {
        let w = generate("alice");
        let kp = keypair_from_stored(&w).unwrap();
        assert_eq!(kp.pubkey().to_string(), w.address);
    }
}
