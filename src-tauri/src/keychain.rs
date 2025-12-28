use keyring::Entry;

const SERVICE: &str = "game_manager";

fn entry_for(key_name: &str) -> Result<Entry, String> {
    // account = key_name, service = constant do app
    Entry::new(SERVICE, key_name).map_err(|e| format!("Erro ao inicializar keychain: {e}"))
}

pub fn set_secret(key_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("Chave não pode ser vazia".to_string());
    }
    entry_for(key_name)?
        .set_password(value)
        .map_err(|e| format!("Erro ao salvar no keychain: {e}"))
}

pub fn get_secret(key_name: &str) -> Result<String, String> {
    match entry_for(key_name)?.get_password() {
        Ok(v) => Ok(v),
        Err(keyring::Error::NoEntry) => Ok(String::new()),
        Err(e) => Err(format!("Erro ao ler do keychain: {e}")),
    }
}

pub fn delete_secret(key_name: &str) -> Result<(), String> {
    // keyring 3.x usa "delete_credential" para remover.
    match entry_for(key_name)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Erro ao remover do keychain: {e}")),
    }
}

pub fn list_supported_keys() -> Vec<&'static str> {
    vec!["steam_id", "steam_api_key", "rawg_api_key"]
}
