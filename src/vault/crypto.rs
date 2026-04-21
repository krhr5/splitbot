use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{Context, Result, anyhow};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::Rng;
use zeroize::Zeroize;

pub const MAGIC: &[u8; 4] = b"SNTV";
pub const VERSION: u8 = 1;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const HEADER_LEN: usize = 4 + 1 + SALT_LEN + NONCE_LEN;

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let params =
        Params::new(64 * 1024, 3, 1, Some(32)).map_err(|e| anyhow!("argon2 params: {e}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow!("argon2 derive: {e}"))?;
    Ok(key)
}

pub fn seal(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let mut rng = rand::rng();
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce_bytes);

    let mut key_bytes = derive_key(passphrase, &salt)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow!("encrypt: {e}"))?;
    key_bytes.zeroize();

    let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn open(blob: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    if blob.len() < HEADER_LEN {
        return Err(anyhow!("vault blob too short"));
    }
    if &blob[0..4] != MAGIC {
        return Err(anyhow!("vault magic mismatch"));
    }
    if blob[4] != VERSION {
        return Err(anyhow!("unsupported vault version {}", blob[4]));
    }
    let salt = &blob[5..5 + SALT_LEN];
    let nonce_bytes = &blob[5 + SALT_LEN..HEADER_LEN];
    let ciphertext = &blob[HEADER_LEN..];

    let mut key_bytes = derive_key(passphrase, salt)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow!("decrypt failed (wrong passphrase or corrupted vault)"))
        .context("open vault")?;
    key_bytes.zeroize();
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let pt = b"hello splitnow";
        let blob = seal(pt, "correct horse").unwrap();
        let recovered = open(&blob, "correct horse").unwrap();
        assert_eq!(recovered, pt);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let blob = seal(b"secret", "one").unwrap();
        assert!(open(&blob, "two").is_err());
    }
}
