//! 密钥安全存储：优先 Windows 凭据管理器（keyring），凭据系统不可用时降级为 AES-256 文件。

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use liteai_core::{SecretError, SecretStore};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Windows 凭据管理器实现（DPAPI 底层）。
pub struct KeyringStore {
    service: String,
}

impl KeyringStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self { service: service.into() }
    }

    fn entry(&self, key: &str) -> Result<keyring::Entry, SecretError> {
        keyring::Entry::new(&self.service, key).map_err(|e| SecretError::Unavailable(e.to_string()))
    }
}

impl SecretStore for KeyringStore {
    fn set(&self, key: &str, value: &str) -> Result<(), SecretError> {
        self.entry(key)?.set_password(value).map_err(|e| SecretError::Other(e.to_string()))
    }
    fn get(&self, key: &str) -> Result<Option<String>, SecretError> {
        match self.entry(key)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretError::Other(e.to_string())),
        }
    }
    fn delete(&self, key: &str) -> Result<(), SecretError> {
        match self.entry(key)?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecretError::Other(e.to_string())),
        }
    }
}

/// AES-256-GCM 文件回退实现。密钥由「机器名 + 用户名 + 应用盐」经 SHA-256 派生。
pub struct FileStore {
    dir: std::path::PathBuf,
}

impl FileStore {
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn secrets_file(&self) -> std::path::PathBuf {
        self.dir.join("secrets.json")
    }

    fn key() -> [u8; 32] {
        let machine = std::env::var("COMPUTERNAME").unwrap_or_default();
        let user = std::env::var("USERNAME").unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(b"liteai-analyzer-v1");
        hasher.update(machine.as_bytes());
        hasher.update(user.as_bytes());
        hasher.finalize().into()
    }

    fn load_map(&self) -> Result<std::collections::HashMap<String, String>, SecretError> {
        let path = self.secrets_file();
        if !path.exists() {
            return Ok(std::collections::HashMap::new());
        }
        let raw = std::fs::read(&path).map_err(|e| SecretError::Other(e.to_string()))?;
        let cipher = Aes256Gcm::new_from_slice(&Self::key()).map_err(|e| SecretError::Other(e.to_string()))?;
        // 存储格式：base64( 12字节nonce || ciphertext )
        let blob = base64::engine::general_purpose::STANDARD
            .decode(raw)
            .map_err(|e| SecretError::Other(format!("解密失败: {e}")))?;
        if blob.len() < 12 {
            return Err(SecretError::Other("secrets.json 损坏".into()));
        }
        let (nonce_bytes, ct) = blob.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let json = cipher.decrypt(nonce, ct).map_err(|_| SecretError::Other("解密失败：密钥不匹配？".into()))?;
        serde_json::from_slice(&json).map_err(|e| SecretError::Other(e.to_string()))
    }

    fn save_map(&self, map: &std::collections::HashMap<String, String>) -> Result<(), SecretError> {
        std::fs::create_dir_all(&self.dir).map_err(|e| SecretError::Other(e.to_string()))?;
        let cipher = Aes256Gcm::new_from_slice(&Self::key()).map_err(|e| SecretError::Other(e.to_string()))?;
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let json = serde_json::to_vec(map).map_err(|e| SecretError::Other(e.to_string()))?;
        let ct = cipher.encrypt(nonce, json.as_ref()).map_err(|e| SecretError::Other(e.to_string()))?;
        let mut blob = Vec::with_capacity(12 + ct.len());
        blob.extend_from_slice(&nonce_bytes);
        blob.extend_from_slice(&ct);
        let encoded = base64::engine::general_purpose::STANDARD.encode(&blob);
        std::fs::write(self.secrets_file(), encoded).map_err(|e| SecretError::Other(e.to_string()))
    }
}

impl SecretStore for FileStore {
    fn set(&self, key: &str, value: &str) -> Result<(), SecretError> {
        let mut map = self.load_map()?;
        map.insert(key.to_string(), value.to_string());
        self.save_map(&map)
    }
    fn get(&self, key: &str) -> Result<Option<String>, SecretError> {
        Ok(self.load_map()?.get(key).cloned())
    }
    fn delete(&self, key: &str) -> Result<(), SecretError> {
        let mut map = self.load_map()?;
        map.remove(key);
        self.save_map(&map)
    }
}

/// 选择密钥存储：凭据管理器可用则用 keyring，否则降级文件存储。
pub fn default_secret_store(config_dir: &Path) -> Box<dyn SecretStore> {
    let keyring = KeyringStore::new(crate::store::SERVICE_NAME);
    match keyring.get("__probe__") {
        Ok(_) | Err(SecretError::Other(_)) => Box::new(keyring),
        Err(SecretError::Unavailable(_)) => Box::new(FileStore::new(config_dir)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_store_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileStore::new(dir.path());
        store.set("api_key", "sk-secret-123").unwrap();
        assert_eq!(store.get("api_key").unwrap().as_deref(), Some("sk-secret-123"));
        store.delete("api_key").unwrap();
        assert_eq!(store.get("api_key").unwrap(), None);
    }
}
