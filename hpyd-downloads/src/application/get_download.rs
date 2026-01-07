use std::sync::Arc;

use hpyd_errors::AppError;
use uuid::Uuid;

use crate::domain::{Download, DownloadRepository};

#[derive(Debug)]
pub struct GetDownload<R: DownloadRepository> {
    repository: Arc<R>,
}

impl<R: DownloadRepository> GetDownload<R> {
    pub const fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(self: Arc<Self>, id: Uuid) -> Result<Download, AppError> {
        tracing::debug!(download_id = %id, "Fetching download");

        self.repository
            .find_by_id(&id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Download {id} not found")))
    }
}
