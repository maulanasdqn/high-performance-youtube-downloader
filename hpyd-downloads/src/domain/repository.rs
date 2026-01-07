use async_trait::async_trait;
use uuid::Uuid;

use super::Download;

#[async_trait]
pub trait DownloadRepository: Send + Sync {
    async fn create(&self, download: Download) -> anyhow::Result<Download>;
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Download>>;
    async fn find_all(&self, limit: u32, offset: u32) -> anyhow::Result<Vec<Download>>;
    async fn count_all(&self) -> anyhow::Result<u64>;
    async fn update(&self, download: Download) -> anyhow::Result<Download>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<()>;
}

