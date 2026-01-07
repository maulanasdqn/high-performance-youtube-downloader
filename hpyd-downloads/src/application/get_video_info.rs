use std::process::Stdio;
use std::sync::Arc;

use hpyd_errors::AppError;
use regex::Regex;
use tokio::process::Command;

use crate::domain::{VideoFormat, VideoInfo, VideoQuality};

#[derive(Debug)]
pub struct GetVideoInfo;

#[derive(Debug, serde::Deserialize)]
struct YtDlpInfo {
    id: String,
    title: String,
    description: Option<String>,
    channel: Option<String>,
    channel_id: Option<String>,
    duration: Option<f64>,
    view_count: Option<u64>,
    like_count: Option<u64>,
    thumbnail: Option<String>,
    upload_date: Option<String>,
    formats: Option<Vec<YtDlpFormat>>,
}

#[derive(Debug, serde::Deserialize)]
struct YtDlpFormat {
    format_id: String,
    ext: String,
    height: Option<u64>,
    filesize: Option<u64>,
    filesize_approx: Option<u64>,
    vcodec: Option<String>,
    acodec: Option<String>,
}

impl GetVideoInfo {
    fn extract_video_id(url_str: &str) -> Result<String, AppError> {
        let patterns = [
            r"(?:youtube\.com/watch\?.*v=|youtube\.com/watch\?v=)([a-zA-Z0-9_-]{11})",
            r"youtu\.be/([a-zA-Z0-9_-]{11})",
            r"youtube\.com/embed/([a-zA-Z0-9_-]{11})",
            r"youtube\.com/v/([a-zA-Z0-9_-]{11})",
            r"youtube\.com/shorts/([a-zA-Z0-9_-]{11})",
            r"^([a-zA-Z0-9_-]{11})$",
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(url_str) {
                    if let Some(video_id) = captures.get(1) {
                        return Ok(video_id.as_str().to_string());
                    }
                }
            }
        }

        Err(AppError::InvalidUrl(
            "Could not extract video ID from URL. Please provide a valid YouTube URL.".to_string(),
        ))
    }

    const fn quality_from_height(height: u64) -> VideoQuality {
        match height {
            h if h >= 2160 => VideoQuality::Quality2160p,
            h if h >= 1440 => VideoQuality::Quality1440p,
            h if h >= 1080 => VideoQuality::Quality1080p,
            h if h >= 720 => VideoQuality::Quality720p,
            h if h >= 480 => VideoQuality::Quality480p,
            _ => VideoQuality::Quality360p,
        }
    }

    fn check_ytdlp() -> Result<(), AppError> {
        which::which("yt-dlp").map_err(|_| {
            AppError::InternalError(
                "yt-dlp not found. Please install yt-dlp (e.g., `nix develop` or `brew install yt-dlp`)".to_string(),
            )
        })?;
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    pub async fn execute(_: Arc<Self>, url: &str) -> Result<VideoInfo, AppError> {
        Self::check_ytdlp()?;

        let video_id_str = Self::extract_video_id(url)?;
        let video_url = format!("https://www.youtube.com/watch?v={video_id_str}");

        tracing::info!(video_id = %video_id_str, "Fetching video info using yt-dlp");

        let output = Command::new("yt-dlp")
            .args([
                "--dump-json",
                "--no-download",
                "--no-warnings",
                &video_url,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to run yt-dlp: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::VideoUnavailable(format!(
                "yt-dlp failed: {stderr}"
            )));
        }

        let info: YtDlpInfo = serde_json::from_slice(&output.stdout)
            .map_err(|e| AppError::InternalError(format!("Failed to parse yt-dlp output: {e}")))?;

        let mut available_formats: Vec<VideoFormat> = Vec::new();
        let mut seen_qualities: std::collections::HashSet<String> = std::collections::HashSet::new();

        if let Some(formats) = &info.formats {
            for fmt in formats {
                let has_video = fmt.vcodec.as_ref().is_some_and(|v| v != "none");
                let has_audio = fmt.acodec.as_ref().is_some_and(|a| a != "none");

                if has_video {
                    if let Some(height) = fmt.height {
                        let quality = Self::quality_from_height(height);
                        let quality_key = if has_audio {
                            quality.to_string()
                        } else {
                            format!("{quality}_video_only")
                        };

                        if seen_qualities.insert(quality_key) {
                            available_formats.push(VideoFormat {
                                quality,
                                format_id: fmt.format_id.clone(),
                                extension: fmt.ext.clone(),
                                file_size: fmt.filesize.or(fmt.filesize_approx),
                                has_video: true,
                                has_audio,
                            });
                        }
                    }
                } else if has_audio && seen_qualities.insert("audio".to_string()) {
                    available_formats.push(VideoFormat {
                        quality: VideoQuality::AudioOnly,
                        format_id: fmt.format_id.clone(),
                        extension: fmt.ext.clone(),
                        file_size: fmt.filesize.or(fmt.filesize_approx),
                        has_video: false,
                        has_audio: true,
                    });
                }
            }
        }

        available_formats.sort_by(|a, b| {
            let order = |q: &VideoQuality| match q {
                VideoQuality::Best => 0,
                VideoQuality::Quality2160p => 1,
                VideoQuality::Quality1440p => 2,
                VideoQuality::Quality1080p => 3,
                VideoQuality::Quality720p => 4,
                VideoQuality::Quality480p => 5,
                VideoQuality::Quality360p => 6,
                VideoQuality::AudioOnly => 7,
            };
            order(&a.quality).cmp(&order(&b.quality))
        });

        if available_formats.is_empty() {
            available_formats.push(VideoFormat {
                quality: VideoQuality::Best,
                format_id: "best".to_string(),
                extension: "mp4".to_string(),
                file_size: None,
                has_video: true,
                has_audio: true,
            });
        }

        let upload_date = info.upload_date.and_then(|date_str| {
            if date_str.len() == 8 {
                let year: i32 = date_str[0..4].parse().ok()?;
                let month: u32 = date_str[4..6].parse().ok()?;
                let day: u32 = date_str[6..8].parse().ok()?;
                chrono::NaiveDate::from_ymd_opt(year, month, day)
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc))
            } else {
                None
            }
        });

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let duration_seconds = info.duration.map_or(0, |d| d as u64);

        Ok(VideoInfo {
            video_id: info.id,
            title: info.title,
            description: info.description,
            channel: info.channel.unwrap_or_else(|| "Unknown".to_string()),
            channel_id: info.channel_id.unwrap_or_default(),
            duration_seconds,
            view_count: info.view_count,
            like_count: info.like_count,
            thumbnail_url: info.thumbnail,
            upload_date,
            available_formats,
        })
    }
}
