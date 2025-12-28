use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand_core::{OsRng, TryRngCore};
use std::fs;
use std::path::PathBuf;

/// Estrutura que representa dados criptografados
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct EncryptedData {
    /// Dados criptografados em base64
    pub ciphertext: String,
    /// Nonce usado na criptografia (12 bytes em base64)
    pub nonce: String,
    /// Salt usado para derivar a chave (16 bytes em base64)
    pub salt: String,
}

/// Gera ou carrega o salt mestre da aplicação
/// Este salt é usado para derivar chaves de criptografia únicas por dispositivo
fn get_master_salt(app_data_dir: &PathBuf) -> Result<Vec<u8>, String> {
    let salt_path = app_data_dir.join(".salt");

    if salt_path.exists() {
        // Carrega salt existente
        let salt = fs::read(&salt_path).map_err(|e| format!("Erro ao ler salt: {}", e))?;

        // Valida tamanho esperado (32 bytes). Se estiver corrompido, falha com erro claro.
        if salt.len() != 32 {
            return Err("Salt mestre inválido: tamanho inesperado".to_string());
        }

        Ok(salt)
    } else {
        // Gera novo salt
        let mut salt = vec![0u8; 32];
        OsRng
            .try_fill_bytes(&mut salt)
            .map_err(|e| format!("Erro ao gerar salt: {}", e))?;

        fs::write(&salt_path, &salt).map_err(|e| format!("Erro ao salvar salt: {}", e))?;

        Ok(salt)
    }
}

/// Deriva uma chave de criptografia a partir de um identificador e salt
/// Usa Argon2id para derivação segura de chave
fn derive_key(identifier: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    // Deriva diretamente 32 bytes (evita parse de PHC string e temporários)
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];

    argon2
        .hash_password_into(identifier.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Erro ao derivar chave: {}", e))?;

    Ok(key)
}

/// Criptografa dados usando AES-256-GCM
pub fn encrypt(
    plaintext: &str,
    app_data_dir: &PathBuf,
    identifier: &str,
) -> Result<EncryptedData, String> {
    // Obtém salt mestre
    let master_salt = get_master_salt(app_data_dir)?;

    // Gera salt único para este dado
    let mut data_salt = [0u8; 16];
    OsRng
        .try_fill_bytes(&mut data_salt)
        .map_err(|e| format!("Erro ao gerar salt do dado: {}", e))?;

    // Combina salt mestre com salt do dado para máxima segurança
    let combined_salt = [master_salt.as_slice(), data_salt.as_slice()].concat();

    // Deriva chave de criptografia
    let key = derive_key(identifier, &combined_salt)?;

    // Cria cipher
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Erro ao criar cipher: {}", e))?;

    // Gera nonce aleatório (12 bytes para GCM)
    let mut nonce_bytes = [0u8; 12];
    OsRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| format!("Erro ao gerar nonce: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Criptografa
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Erro ao criptografar: {}", e))?;

    Ok(EncryptedData {
        ciphertext: BASE64.encode(&ciphertext),
        nonce: BASE64.encode(&nonce_bytes),
        salt: BASE64.encode(&data_salt),
    })
}

/// Descriptografa dados usando AES-256-GCM
pub fn decrypt(
    encrypted: &EncryptedData,
    app_data_dir: &PathBuf,
    identifier: &str,
) -> Result<String, String> {
    // Obtém salt mestre
    let master_salt = get_master_salt(app_data_dir)?;

    // Decodifica salt dos dados
    let data_salt = BASE64
        .decode(&encrypted.salt)
        .map_err(|e| format!("Erro ao decodificar salt: {}", e))?;
    if data_salt.len() != 16 {
        return Err("Salt inválido: esperado 16 bytes".to_string());
    }

    // Combina salts
    let combined_salt = [master_salt.as_slice(), data_salt.as_slice()].concat();

    // Deriva chave
    let key = derive_key(identifier, &combined_salt)?;

    // Cria cipher
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Erro ao criar cipher: {}", e))?;

    // Decodifica nonce
    let nonce_vec = BASE64
        .decode(&encrypted.nonce)
        .map_err(|e| format!("Erro ao decodificar nonce: {}", e))?;
    let nonce_bytes: [u8; 12] = nonce_vec
        .as_slice()
        .try_into()
        .map_err(|_| "Nonce inválido: esperado 12 bytes".to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decodifica ciphertext
    let ciphertext = BASE64
        .decode(&encrypted.ciphertext)
        .map_err(|e| format!("Erro ao decodificar ciphertext: {}", e))?;

    // Descriptografa
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "Erro ao descriptografar: dados corrompidos ou chave inválida".to_string())?;

    String::from_utf8(plaintext).map_err(|e| format!("Erro ao converter para string: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_encrypt_decrypt() {
        let temp_dir = env::temp_dir();
        let test_data = "minha_api_key_secreta_123";
        let identifier = "test_key";

        let encrypted = encrypt(test_data, &temp_dir, identifier).unwrap();
        let decrypted = decrypt(&encrypted, &temp_dir, identifier).unwrap();

        assert_eq!(test_data, decrypted);
    }

    #[test]
    fn test_different_identifiers_different_results() {
        let temp_dir = env::temp_dir();
        let test_data = "same_data";

        let encrypted1 = encrypt(test_data, &temp_dir, "id1").unwrap();
        let encrypted2 = encrypt(test_data, &temp_dir, "id2").unwrap();

        // Mesmos dados, identificadores diferentes = ciphertexts diferentes
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }
}