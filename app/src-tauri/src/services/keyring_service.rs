use keyring::Entry;
use crate::models::AppError;

const SERVICE_NAME: &str = "ai-skills-aggregator";

pub struct KeyringService;

impl KeyringService {
    pub fn store_token(username: &str, token: &str) -> Result<(), AppError> {
        let entry = Entry::new(SERVICE_NAME, username)
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.set_password(token)
            .map_err(|e| AppError::IoError(format!("Failed to store token: {}", e)))
    }

    pub fn get_token(username: &str) -> Result<String, AppError> {
        let entry = Entry::new(SERVICE_NAME, username)
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.get_password()
            .map_err(|e| AppError::IoError(format!("Failed to get token: {}", e)))
    }

    pub fn delete_token(username: &str) -> Result<(), AppError> {
        let entry = Entry::new(SERVICE_NAME, username)
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.delete_credential()
            .map_err(|e| AppError::IoError(format!("Failed to delete token: {}", e)))
    }

    pub fn store_current_user(username: &str) -> Result<(), AppError> {
        let entry = Entry::new(SERVICE_NAME, "current_user")
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.set_password(username)
            .map_err(|e| AppError::IoError(format!("Failed to store user: {}", e)))
    }

    pub fn get_current_user() -> Result<String, AppError> {
        let entry = Entry::new(SERVICE_NAME, "current_user")
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.get_password()
            .map_err(|e| AppError::IoError(format!("No stored user: {}", e)))
    }

    pub fn clear_current_user() -> Result<(), AppError> {
        let entry = Entry::new(SERVICE_NAME, "current_user")
            .map_err(|e| AppError::IoError(format!("Keyring error: {}", e)))?;

        entry.delete_credential()
            .map_err(|e| AppError::IoError(format!("Failed to clear user: {}", e)))
    }
}
