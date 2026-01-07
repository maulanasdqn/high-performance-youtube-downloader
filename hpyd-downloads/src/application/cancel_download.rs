use std::sync::Arc;

use hpyd_errors::AppError;
use uuid::Uuid;

use crate::domain::{Download, DownloadRepository, DownloadStatus};

#[derive(Debug)]
pub struct CancelDownload<R: DownloadRepository> {
    repository: Arc<R>,
}

impl<R: DownloadRepository> CancelDownload<R> {
    pub const fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(self: Arc<Self>, id: Uuid) -> Result<Download, AppError> {
        tracing::info!(download_id = %id, "Cancelling download");

        let mut download = self
            .repository
            .find_by_id(&id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Download {id} not found")))?;

        if download.is_terminal() {
            return Err(AppError::BadRequest(format!(
                "Cannot cancel download with status: {:?}",
                download.status
            )));
        }

        if download.status == DownloadStatus::Cancelled {
            return Err(AppError::BadRequest("Download already cancelled".to_string()));
        }

        download.cancel();

        self.repository
            .update(download)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }
}
