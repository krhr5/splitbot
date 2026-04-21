pub mod crypto;
pub mod model;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub use model::{ChainFamily, StoredWallet, VaultFile, WalletSecret};

pub struct Vault {
    path: PathBuf,
    passphrase: String,
    pub data: VaultFile,
}

impl Vault {
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    pub fn create(path: PathBuf, passphrase: String, api_key: String) -> Result<Self> {
        let data = VaultFile {
            api_key: Some(api_key),
            wallets: Vec::new(),
        };
        let v = Self {
            path,
            passphrase,
            data,
        };
        v.save()?;
        Ok(v)
    }

    pub fn unlock(path: PathBuf, passphrase: String) -> Result<Self> {
        let blob = fs::read(&path).with_context(|| format!("read vault {}", path.display()))?;
        let plaintext = crypto::open(&blob, &passphrase)?;
        let data: VaultFile = serde_json::from_slice(&plaintext).context("parse vault json")?;
        Ok(Self {
            path,
            passphrase,
            data,
        })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
        }
        let plaintext = serde_json::to_vec(&self.data).context("serialize vault")?;
        let blob = crypto::seal(&plaintext, &self.passphrase)?;
        let tmp = self.path.with_extension("bin.tmp");
        fs::write(&tmp, &blob).with_context(|| format!("write {}", tmp.display()))?;
        fs::rename(&tmp, &self.path)
            .with_context(|| format!("rename into {}", self.path.display()))?;
        Ok(())
    }

    pub fn api_key(&self) -> Option<&str> {
        self.data.api_key.as_deref()
    }

    pub fn set_api_key(&mut self, api_key: String) -> Result<()> {
        self.data.api_key = Some(api_key);
        self.save()
    }

    pub fn add_wallet(&mut self, w: StoredWallet) -> Result<()> {
        self.data.wallets.push(w);
        self.save()
    }

    pub fn remove_wallet(&mut self, index: usize) -> Result<()> {
        if index < self.data.wallets.len() {
            self.data.wallets.remove(index);
            self.save()?;
        }
        Ok(())
    }

    pub fn rename_wallet(&mut self, index: usize, label: String) -> Result<()> {
        if let Some(wallet) = self.data.wallets.get_mut(index) {
            wallet.label = label;
            self.save()?;
        }
        Ok(())
    }

    pub fn wallets(&self) -> &[StoredWallet] {
        &self.data.wallets
    }
}
