use crate::commands::libraries::core::spawn_import;
use crate::errors::AppError;
use tauri::AppHandle;

#[tauri::command]
pub async fn import_ubisoft_games(
    app: AppHandle,
    wine_prefix: Option<String>,
) -> Result<(), AppError> {
    use crate::providers::libraries::providers::GameSource;
    use crate::providers::libraries::ubisoft::UbisoftSource;

    let prefix = wine_prefix
        .filter(|s| !s.trim().is_empty())
        .map(std::path::PathBuf::from);

    spawn_import(app, "Ubisoft", |_app| async move {
        UbisoftSource::new(true, prefix).fetch_games().await
    });
    Ok(())
}
