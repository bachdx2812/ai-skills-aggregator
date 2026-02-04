use crate::models::User;
use crate::services::auth_service::AuthService;

#[tauri::command]
pub async fn login(app: tauri::AppHandle) -> Result<User, String> {
    let auth = AuthService::new();

    // Get auth URL
    let auth_url = auth.start_login()
        .await
        .map_err(|e| e.to_string())?;

    // Open in system browser
    tauri::async_runtime::spawn(async move {
        if let Err(e) = tauri_plugin_shell::ShellExt::shell(&app)
            .open(&auth_url, None) {
            log::error!("Failed to open browser: {}", e);
        }
    });

    // Wait for callback
    auth.wait_for_callback()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_current_user() -> Result<Option<User>, String> {
    let auth = AuthService::new();
    match auth.get_current_user().await {
        Ok(user) => Ok(Some(user)),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn logout() -> Result<(), String> {
    let auth = AuthService::new();
    auth.logout().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_logged_in() -> bool {
    let auth = AuthService::new();
    auth.is_logged_in()
}
