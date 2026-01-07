use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::VideoQuality;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    #[default]
    Pending,
    Fetching,
    Downloading,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Download {
    pub id: Uuid,
    pub video_id: String,
    pub video_url: String,
    pub title: String,
    pub quality: VideoQuality,
    pub status: DownloadStatus,
    pub progress: f32,
    pub file_path: Option<String>,
    pub file_size: Option<u64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Download {
    #[must_use]
    pub fn new(video_id: String, video_url: String, title: String, quality: VideoQuality) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            video_id,
            video_url,
            title,
            quality,
            status: DownloadStatus::Pending,
            progress: 0.0,
            file_path: None,
            file_size: None,
            error_message: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    pub fn start_fetching(&mut self) {
        self.status = DownloadStatus::Fetching;
        self.updated_at = Utc::now();
    }

    pub fn start_downloading(&mut self) {
        self.status = DownloadStatus::Downloading;
        self.updated_at = Utc::now();
    }

    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 100.0);
        self.updated_at = Utc::now();
    }

    pub fn start_processing(&mut self) {
        self.status = DownloadStatus::Processing;
        self.updated_at = Utc::now();
    }

    pub fn complete(&mut self, file_path: String, file_size: u64) {
        self.status = DownloadStatus::Completed;
        self.progress = 100.0;
        self.file_path = Some(file_path);
        self.file_size = Some(file_size);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn fail(&mut self, error_message: String) {
        self.status = DownloadStatus::Failed;
        self.error_message = Some(error_message);
        self.updated_at = Utc::now();
    }

    pub fn cancel(&mut self) {
        self.status = DownloadStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Cancelled
        )
    }
}
