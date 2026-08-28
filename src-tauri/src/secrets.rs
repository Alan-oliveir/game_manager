use once_cell::sync::OnceCell;
use sha2::{Digest, Sha256};
use tauri::AppHandle;

static MASTER_KEY: OnceCell<[u8; 32]> = OnceCell::new();

// Retorna a chave mestre derivada de dados únicos da máquina.
pub fn master_key(app: &AppHandle) -> Result<&'static [u8; 32], String> {
    MASTER_KEY.get_or_try_init(|| {
        let uid = machine_uid::get().map_err(|e| e.to_string())?;

        let username = whoami::username().unwrap_or_else(|_| "unknown_user".to_string());
        let app_id = app.config().identifier.clone();

        let mut hasher = Sha256::new();
        hasher.update(uid);
        hasher.update(username.as_bytes());
        hasher.update(app_id.as_bytes());

        let hash = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash[..32]);
        Ok(key)
    })
}
