use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

// NOTA IMPORTANTE:
// Em um app comercial real, usariamos keyrings ou outro metodo seguro para armazenar a chave mestre.
// Para este projeto desktop standalone, usamos uma chave obfuscada.
const MASTER_KEY: &[u8; 32] = b"Playlite-fghjjkllzxcvbn4567890ab"; // 32 bytes para AES-256

/// Encripta uma string e retorna o resultado em Hex
pub fn encrypt(data: &str) -> String {
    let cipher = Aes256Gcm::new(MASTER_KEY.into());

    // Gera um Nonce (número usado uma vez) aleatório
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encripta
    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .expect("falha na encriptação"); // Em portfólio panic aqui é aceitável se a memória falhar

    // Combina Nonce + Ciphertext para salvar junto
    let mut final_msg = nonce_bytes.to_vec();
    final_msg.extend_from_slice(&ciphertext);

    hex::encode(final_msg)
}

/// Decripta uma string Hex para o texto original
pub fn decrypt(encoded_data: &str) -> Result<String, String> {
    let data = hex::decode(encoded_data).map_err(|_| "Falha ao decodificar Hex")?;

    if data.len() < 12 {
        return Err("Dados corrompidos ou muito curtos".to_string());
    }

    // Separa o Nonce do conteúdo
    let (nonce_arr, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(MASTER_KEY.into());
    let nonce = Nonce::from_slice(nonce_arr);

    // Decripta
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Senha incorreta ou dados corrompidos")?;

    String::from_utf8(plaintext).map_err(|_| "Erro de codificação UTF-8".to_string())
}
