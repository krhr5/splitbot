use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Clone, Serialize)]
pub struct StoredWallet {
    pub label: String,
    pub chain_family: ChainFamily,
    pub address: String,
    pub secret: WalletSecret,
}

impl fmt::Debug for StoredWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoredWallet")
            .field("label", &self.label)
            .field("chain_family", &self.chain_family)
            .field("address", &self.address)
            .field("secret", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChainFamily {
    #[default]
    Solana,
    Evm,
    Bitcoin,
    Monero,
}

impl ChainFamily {
    pub const ALL: [Self; 4] = [Self::Solana, Self::Evm, Self::Bitcoin, Self::Monero];

    pub fn label(self) -> &'static str {
        match self {
            Self::Solana => "Solana",
            Self::Evm => "EVM",
            Self::Bitcoin => "Bitcoin",
            Self::Monero => "Monero",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WalletSecret {
    Solana {
        #[serde(with = "secret_64_serde")]
        bytes: [u8; 64],
    },
    Evm {
        private_key_hex: String,
    },
    Bitcoin {
        wif: String,
    },
    Monero {
        private_spend_key_hex: String,
        private_view_key_hex: String,
    },
}

impl fmt::Debug for WalletSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Solana { .. } => f
                .debug_struct("Solana")
                .field("bytes", &"[redacted]")
                .finish(),
            Self::Evm { .. } => f
                .debug_struct("Evm")
                .field("private_key_hex", &"[redacted]")
                .finish(),
            Self::Bitcoin { .. } => f
                .debug_struct("Bitcoin")
                .field("wif", &"[redacted]")
                .finish(),
            Self::Monero { .. } => f
                .debug_struct("Monero")
                .field("private_spend_key_hex", &"[redacted]")
                .field("private_view_key_hex", &"[redacted]")
                .finish(),
        }
    }
}

impl<'de> Deserialize<'de> for StoredWallet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WireStoredWallet {
            Current {
                label: String,
                chain_family: ChainFamily,
                address: String,
                secret: WalletSecret,
            },
            LegacySolana {
                label: String,
                pubkey: String,
                #[serde(with = "secret_64_serde")]
                secret_bytes: [u8; 64],
            },
        }

        match WireStoredWallet::deserialize(deserializer)? {
            WireStoredWallet::Current {
                label,
                chain_family,
                address,
                secret,
            } => Ok(Self {
                label,
                chain_family,
                address,
                secret,
            }),
            WireStoredWallet::LegacySolana {
                label,
                pubkey,
                secret_bytes,
            } => Ok(Self {
                label,
                chain_family: ChainFamily::Solana,
                address: pubkey,
                secret: WalletSecret::Solana {
                    bytes: secret_bytes,
                },
            }),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct VaultFile {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub wallets: Vec<StoredWallet>,
}

impl fmt::Debug for VaultFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VaultFile")
            .field("api_key", &self.api_key.as_ref().map(|_| "[redacted]"))
            .field("wallets", &self.wallets)
            .finish()
    }
}

mod secret_64_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 64], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 64], D::Error> {
        let v = <Vec<u8>>::deserialize(d)?;
        v.try_into()
            .map_err(|v: Vec<u8>| serde::de::Error::invalid_length(v.len(), &"64 bytes"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_legacy_solana_wallet() {
        let json = r#"{
            "label": "old-clean",
            "pubkey": "LegacyPubkey",
            "secret_bytes": [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
        }"#;

        let wallet: StoredWallet = serde_json::from_str(json).unwrap();
        assert_eq!(wallet.label, "old-clean");
        assert_eq!(wallet.chain_family, ChainFamily::Solana);
        assert_eq!(wallet.address, "LegacyPubkey");
        let WalletSecret::Solana { bytes } = wallet.secret else {
            panic!("expected Solana secret");
        };
        assert_eq!(bytes, [1u8; 64]);
    }
}
