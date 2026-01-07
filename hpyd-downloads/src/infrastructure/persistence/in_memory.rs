use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{Download, DownloadRepository};

#[derive(Debug, Default)]
pub struct InMemoryDownloadRepository {
    downloads: RwLock<HashMap<Uuid, Download>>,
}

impl InMemoryDownloadRepository {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            downloads: RwLock::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl DownloadRepository for InMemoryDownloadRepository {
    async fn create(&self, download: Download) -> anyhow::Result<Download> {
        self.downloads.write().await.insert(download.id, download.clone());
        Ok(download)
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Download>> {
        Ok(self.downloads.read().await.get(id).cloned())
    }

    async fn find_all(&self, limit: u32, offset: u32) -> anyhow::Result<Vec<Download>> {
        let mut all_downloads: Vec<Download> = self.downloads.read().await.values().cloned().collect();

        all_downloads.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let result = all_downloads
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();

        Ok(result)
    }

    async fn count_all(&self) -> anyhow::Result<u64> {
        Ok(self.downloads.read().await.len() as u64)
    }

    async fn update(&self, download: Download) -> anyhow::Result<Download> {
        self.downloads.write().await.insert(download.id, download.clone());
        Ok(download)
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<()> {
        self.downloads.write().await.remove(id);
        Ok(())
    }
}
