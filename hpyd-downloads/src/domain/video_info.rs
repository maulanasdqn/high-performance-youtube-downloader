use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum VideoQuality {
    #[default]
    Best,
    Quality2160p,
    Quality1440p,
    Quality1080p,
    Quality720p,
    Quality480p,
    Quality360p,
    AudioOnly,
}

impl std::fmt::Display for VideoQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Best => write!(f, "best"),
            Self::Quality2160p => write!(f, "2160p"),
            Self::Quality1440p => write!(f, "1440p"),
            Self::Quality1080p => write!(f, "1080p"),
            Self::Quality720p => write!(f, "720p"),
            Self::Quality480p => write!(f, "480p"),
            Self::Quality360p => write!(f, "360p"),
            Self::AudioOnly => write!(f, "audio"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VideoFormat {
    pub quality: VideoQuality,
    pub format_id: String,
    pub extension: String,
    pub file_size: Option<u64>,
    pub has_video: bool,
    pub has_audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VideoInfo {
    pub video_id: String,
    pub title: String,
    pub description: Option<String>,
    pub channel: String,
    pub channel_id: String,
    pub duration_seconds: u64,
    pub view_count: Option<u64>,
    pub like_count: Option<u64>,
    pub thumbnail_url: Option<String>,
    pub upload_date: Option<DateTime<Utc>>,
    pub available_formats: Vec<VideoFormat>,
}

impl VideoInfo {
    #[must_use]
    pub fn duration_formatted(&self) -> String {
        let hours = self.duration_seconds / 3600;
        let minutes = (self.duration_seconds % 3600) / 60;
        let seconds = self.duration_seconds % 60;

        if hours > 0 {
            format!("{hours}:{minutes:02}:{seconds:02}")
        } else {
            format!("{minutes}:{seconds:02}")
        }
    }
}

