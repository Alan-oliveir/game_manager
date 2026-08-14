//! Comandos para abrir pastas e arquivos

use crate::errors::AppError;
use tauri_plugin_opener::OpenerExt;

/// Abre uma pasta no explorador de arquivos do sistema
#[tauri::command]
pub async fn open_folder(app: tauri::AppHandle, path: String) -> Result<(), AppError> {
    // Validar que o caminho existe e é uma pasta
    let path_obj = std::path::Path::new(&path);

    // Cria o diretório se não existir (útil para pastas como analysis)
    if !path_obj.exists() {
        std::fs::create_dir_all(path_obj)
            .map_err(|e| AppError::IoError(format!("Erro ao criar pasta: {}", e)))?;
    }

    if !path_obj.is_dir() {
        return Err(AppError::IoError(format!(
            "O caminho não é uma pasta: {}",
            path
        )));
    }

    // Usar o plugin opener do Tauri
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| AppError::IoError(format!("Erro ao abrir pasta: {}", e)))?;

    Ok(())
}

/// Abre um arquivo com o aplicativo padrão
#[tauri::command]
pub async fn open_file(app: tauri::AppHandle, path: String) -> Result<(), AppError> {
    // Validar que o arquivo existe
    let path_obj = std::path::Path::new(&path);

    if !path_obj.exists() {
        return Err(AppError::NotFound(format!(
            "Arquivo não encontrado: {}",
            path
        )));
    }

    if !path_obj.is_file() {
        return Err(AppError::IoError(format!(
            "O caminho não é um arquivo: {}",
            path
        )));
    }

    // Usar o plugin opener do Tauri
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| AppError::IoError(format!("Erro ao abrir arquivo: {}", e)))?;

    Ok(())
}
