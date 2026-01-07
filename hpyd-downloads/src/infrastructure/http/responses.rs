use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{Download, DownloadStatus, VideoInfo, VideoQuality};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VideoInfoResponse {
    pub video_id: String,
    pub title: String,
    pub description: Option<String>,
    pub channel: String,
    pub channel_id: String,
    pub duration: String,
    pub duration_seconds: u64,
    pub view_count: Option<u64>,
    pub like_count: Option<u64>,
    pub thumbnail_url: Option<String>,
    pub available_qualities: Vec<String>,
}

impl From<VideoInfo> for VideoInfoResponse {
    fn from(info: VideoInfo) -> Self {
        let duration = info.duration_formatted();
        let available_qualities: Vec<String> = info
            .available_formats
            .iter()
            .map(|f| f.quality.to_string())
            .collect();

        Self {
            video_id: info.video_id,
            title: info.title,
            description: info.description,
            channel: info.channel,
            channel_id: info.channel_id,
            duration,
            duration_seconds: info.duration_seconds,
            view_count: info.view_count,
            like_count: info.like_count,
            thumbnail_url: info.thumbnail_url,
            available_qualities,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DownloadResponse {
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

impl From<Download> for DownloadResponse {
    fn from(download: Download) -> Self {
        Self {
            id: download.id,
            video_id: download.video_id,
            video_url: download.video_url,
            title: download.title,
            quality: download.quality,
            status: download.status,
            progress: download.progress,
            file_path: download.file_path,
            file_size: download.file_size,
            error_message: download.error_message,
            created_at: download.created_at,
            updated_at: download.updated_at,
            completed_at: download.completed_at,
        }
    }
}

