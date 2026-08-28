use crate::errors::AppError;
use crate::providers::achievements::xbox::XboxLiveSource;
use crate::services::libraries::spawn_import;
use tauri::AppHandle;

#[tauri::command]
pub async fn xbox_live_login(
    app: AppHandle,
    client_id: String,
    client_secret: String,
) -> Result<String, AppError> {
    let source = XboxLiveSource::new(app, client_id, client_secret);
    source.login().await?;
    Ok("Conta Xbox Live conectada com sucesso!".to_string())
}

#[tauri::command]
pub async fn import_xbox_games(app: AppHandle) -> Result<(), AppError> {
    spawn_import(app, "Xbox", |_app| async move {
        crate::providers::libraries::xbox::import_installed()
    });
    Ok(())
}
