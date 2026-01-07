use std::sync::Arc;

use hpyd_errors::AppError;
use hpyd_types::PaginationMeta;

use crate::domain::{Download, DownloadRepository};

#[derive(Debug)]
pub struct ListDownloads<R: DownloadRepository> {
    repository: Arc<R>,
}

impl<R: DownloadRepository> ListDownloads<R> {
    pub const fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        self: Arc<Self>,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<Download>, PaginationMeta), AppError> {
        let offset = (page - 1) * per_page;

        let downloads = self
            .repository
            .find_all(per_page, offset)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let total = self
            .repository
            .count_all()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let meta = PaginationMeta::new(page, per_page, total);

        Ok((downloads, meta))
    }
}
