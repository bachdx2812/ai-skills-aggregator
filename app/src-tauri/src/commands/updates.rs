use crate::services::update_service::{UpdateService, SkillUpdate, UpdateCheckResult};

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let service = UpdateService::new();
    Ok(service.check_all_updates().await)
}

#[tauri::command]
pub async fn apply_skill_update(update: SkillUpdate) -> Result<(), String> {
    let service = UpdateService::new();
    service.apply_update(&update).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_all_skill_updates(updates: Vec<SkillUpdate>) -> Result<Vec<Result<(), String>>, String> {
    let service = UpdateService::new();
    Ok(service.apply_all_updates(&updates).await)
}

#[tauri::command]
pub async fn rollback_skill(skill_id: String, agent: String) -> Result<(), String> {
    let service = UpdateService::new();
    service.rollback_skill(&skill_id, &agent).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn skip_skill_version(skill_id: String, version: String) -> Result<(), String> {
    let service = UpdateService::new();
    service.skip_version(&skill_id, &version).await.map_err(|e| e.to_string())
}
