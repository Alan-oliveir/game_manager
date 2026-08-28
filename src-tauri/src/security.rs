//! Módulo de segurança centralizado.
//!
//! Integra as funcionalidades de:
//! - Derivação de chave mestra (secrets.rs)
//! - Criptografia/descriptografia (crypto.rs)
//!
//! Fornece uma API simplificada para o resto da aplicação.

use crate::errors::AppError;
use crate::{crypto, secrets};
use base64::{engine::general_purpose, Engine as _};
use tauri::AppHandle;

/// Inicializa o sistema de segurança.
///
/// Deve ser chamado durante o setup da aplicação para garantir que a chave mestra
/// seja derivada e armazenada na memória.
///
/// **Erros:**
/// - Se falhar ao obter o Machine UID
/// - Se falhar ao derivar a chave mestra
pub fn init_security(app: &AppHandle) -> Result<(), String> {
    secrets::master_key(app)?;
    Ok(())
}

/// Encripta um valor de texto usando a chave mestra derivada.
///
/// **Parâmetros:**
/// - `app`: Handle da aplicação para acessar a chave mestra
/// - `plaintext`: String em texto claro a ser encriptada
///
/// **Retorna:**
/// - Base64 do payload encriptado (nonce + ciphertext)
pub fn encrypt(app: &AppHandle, plaintext: &str) -> Result<String, AppError> {
    let master_key = secrets::master_key(app).map_err(AppError::DatabaseError)?;
    let encrypted_bytes =
        crypto::encrypt(master_key, plaintext.as_bytes()).map_err(AppError::DatabaseError)?;
    Ok(general_purpose::STANDARD.encode(&encrypted_bytes))
}

/// Descriptografa um valor encriptado usando a chave mestra derivada.
///
/// **Parâmetros:**
/// - `app`: Handle da aplicação para acessar a chave mestra
/// - `encrypted`: String em Base64 do payload encriptado
///
/// **Retorna:**
/// - String em texto claro
///
/// **Erros:**
/// - Se o formato Base64 for inválido
/// - Se a descriptografia falhar (chave incorreta, payload corrompido)
pub fn decrypt(app: &AppHandle, encrypted: &str) -> Result<String, AppError> {
    let master_key = secrets::master_key(app).map_err(AppError::DatabaseError)?;
    let encrypted_bytes = general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let decrypted_bytes =
        crypto::decrypt(master_key, &encrypted_bytes).map_err(AppError::DatabaseError)?;
    String::from_utf8(decrypted_bytes).map_err(|e| AppError::DatabaseError(e.to_string()))
}
