use crate::errors::AppError;
use crate::providers::libraries::ea::EaSource;
use crate::services::libraries::spawn_import;
use tauri::AppHandle;

#[tauri::command]
pub async fn import_ea_games(
    app: AppHandle,
    ea_install_dir: Option<String>,
) -> Result<(), AppError> {
    let install_dir = ea_install_dir
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    spawn_import(app, "EA", |_app| async move {
        EaSource::new(install_dir).import_installed().await
    });
    Ok(())
}
