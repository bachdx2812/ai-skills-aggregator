use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::fs;
use crate::models::AppError;
use serde::Serialize;

pub struct BackupService {
    backup_dir: PathBuf,
    retention_days: u64,
}

impl BackupService {
    pub fn new() -> Self {
        let backup_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai-skills-aggregator")
            .join("backups");

        Self {
            backup_dir,
            retention_days: 7,
        }
    }

    pub async fn backup_file(&self, file_path: &str) -> Result<PathBuf, AppError> {
        let source = PathBuf::from(file_path);
        if !source.exists() {
            return Err(AppError::FileNotFound(file_path.to_string()));
        }

        // Create backup filename with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let file_name = source.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let backup_name = format!("{}_{}", timestamp, file_name);

        // Ensure backup dir exists
        fs::create_dir_all(&self.backup_dir)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let backup_path = self.backup_dir.join(backup_name);

        // Copy file to backup
        fs::copy(&source, &backup_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(backup_path)
    }

    pub async fn backup_folder(&self, folder_path: &str) -> Result<PathBuf, AppError> {
        let source = PathBuf::from(folder_path);
        if !source.exists() {
            return Err(AppError::FileNotFound(folder_path.to_string()));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let folder_name = source.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let backup_name = format!("{}_{}", timestamp, folder_name);

        // Ensure backup dir exists
        fs::create_dir_all(&self.backup_dir)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let backup_path = self.backup_dir.join(backup_name);

        // Copy folder recursively
        self.copy_dir_recursive(&source, &backup_path).await?;

        Ok(backup_path)
    }

    async fn copy_dir_recursive(&self, src: &PathBuf, dst: &PathBuf) -> Result<(), AppError> {
        fs::create_dir_all(dst)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let mut entries = fs::read_dir(src)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?
        {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Box::pin(self.copy_dir_recursive(&src_path, &dst_path)).await?;
            } else {
                fs::copy(&src_path, &dst_path)
                    .await
                    .map_err(|e| AppError::IoError(e.to_string()))?;
            }
        }

        Ok(())
    }

    pub async fn restore_file(&self, backup_path: &str, dest_path: &str) -> Result<(), AppError> {
        let backup = PathBuf::from(backup_path);
        let dest = PathBuf::from(dest_path);

        if !backup.exists() {
            return Err(AppError::FileNotFound(backup_path.to_string()));
        }

        // Ensure parent dir exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        }

        fs::copy(&backup, &dest)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(())
    }

    pub async fn cleanup_old_backups(&self) -> Result<usize, AppError> {
        if !self.backup_dir.exists() {
            return Ok(0);
        }

        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(self.retention_days * 24 * 60 * 60))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let mut deleted = 0;

        let mut entries = fs::read_dir(&self.backup_dir)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::IoError(e.to_string()))? {

            let metadata = entry.metadata().await
                .map_err(|e| AppError::IoError(e.to_string()))?;

            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    let path = entry.path();
                    if path.is_dir() {
                        if fs::remove_dir_all(&path).await.is_ok() {
                            deleted += 1;
                        }
                    } else if fs::remove_file(&path).await.is_ok() {
                        deleted += 1;
                    }
                }
            }
        }

        Ok(deleted)
    }

    pub async fn list_backups(&self, original_filename: &str) -> Result<Vec<BackupInfo>, AppError> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let mut entries = fs::read_dir(&self.backup_dir)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::IoError(e.to_string()))? {

            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(original_filename) {
                if let Ok(metadata) = entry.metadata().await {
                    let size = metadata.len();
                    let modified = metadata.modified()
                        .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
                        .unwrap_or(0);

                    backups.push(BackupInfo {
                        path: entry.path().to_string_lossy().to_string(),
                        name,
                        size,
                        created_at: modified,
                    });
                }
            }
        }

        // Sort by creation time descending
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }
}

impl Default for BackupService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub created_at: i64,
}
