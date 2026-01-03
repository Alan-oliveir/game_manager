use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use once_cell::sync::OnceCell;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs;
use tauri::{AppHandle, Manager};
use tauri_plugin_machine_uid::MachineUidExt;

static MASTER_KEY: OnceCell<[u8; 32]> = OnceCell::new();

const SALT_FILE: &str = "crypto_salt.bin";

/// Inicializa o sistema de segurança derivando a chave mestre dos dados da máquina
/// Deve ser chamado UMA vez na inicialização do app
pub fn init_security(app: &AppHandle) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Não foi possível resolver app_data_dir: {}", e))?;

    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    // Salt persistido
    let salt_path = app_dir.join(SALT_FILE);
    let salt = if salt_path.exists() {
        fs::read(&salt_path).map_err(|e| e.to_string())?
    } else {
        let mut s = vec![0u8; 16];
        rand::rng().fill_bytes(&mut s);
        fs::write(&salt_path, &s).map_err(|e| e.to_string())?;
        s
    };

    // Dados do ambiente
    let machine_uid_result = app
        .machine_uid()
        .get_machine_uid()
        .map_err(|e| e.to_string())?;

    let machine_uid = machine_uid_result
        .id
        .ok_or("Machine UID indisponível")?;

    let username = whoami::username().map_err(|e| e.to_string())?;
    let app_id = app.config().identifier.clone();

    // Derivação da chave usando SHA256
    let mut hasher = Sha256::new();
    hasher.update(machine_uid.as_bytes());
    hasher.update(username.as_bytes());
    hasher.update(app_id.as_bytes());
    hasher.update(salt);

    let key: [u8; 32] = hasher.finalize().into();
    MASTER_KEY
        .set(key)
        .map_err(|_| "Chave já inicializada")?;

    Ok(())
}

fn cipher() -> Aes256Gcm {
    let key = MASTER_KEY
        .get()
        .expect("Security não inicializado. Chame init_security()");
    Aes256Gcm::new_from_slice(key).unwrap()
}

/// Encripta uma string e retorna Base64 (formato: ciphertext_b64:nonce_b64)
pub fn encrypt(data: &str) -> String {
    let cipher = cipher();

    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data.as_bytes()).unwrap();

    format!(
        "{}:{}",
        base64::engine::general_purpose::STANDARD.encode(ciphertext),
        base64::engine::general_purpose::STANDARD.encode(nonce_bytes)
    )
}

/// Decripta uma string Base64 para o texto original
pub fn decrypt(data: &str) -> Result<String, String> {
    let cipher = cipher();

    let parts: Vec<&str> = data.split(':').collect();
    if parts.len() != 2 {
        return Err("Formato inválido".into());
    }

    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(parts[0])
        .map_err(|e| e.to_string())?;
    let nonce_bytes = base64::engine::general_purpose::STANDARD
        .decode(parts[1])
        .map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plain = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| e.to_string())?;

    String::from_utf8(plain).map_err(|e| e.to_string())
}
