use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use hpyd_errors::AppError;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::domain::{Download, DownloadRepository, VideoQuality};
use crate::infrastructure::S3Service;

#[derive(Debug)]
pub struct CreateDownload<R: DownloadRepository + 'static> {
    repository: Arc<R>,
    download_dir: PathBuf,
    s3_service: Option<Arc<S3Service>>,
}

impl<R: DownloadRepository + 'static> CreateDownload<R> {
    pub const fn new(repository: Arc<R>, download_dir: PathBuf, s3_service: Option<Arc<S3Service>>) -> Self {
        Self {
            repository,
            download_dir,
            s3_service,
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

    pub async fn execute(
        self: Arc<Self>,
        video_id: String,
        video_url: String,
        title: String,
        quality: VideoQuality,
    ) -> Result<Download, AppError> {
        Self::check_ytdlp()?;

        tracing::info!(
            video_id = %video_id,
            quality = %quality,
            "Creating new download"
        );

        let download = Download::new(video_id.clone(), video_url.clone(), title.clone(), quality.clone());

        let download = self
            .repository
            .create(download)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let download_id = download.id;
        let repository = self.repository.clone();
        let download_dir = self.download_dir.clone();
        let s3_service = self.s3_service.clone();

        tokio::spawn(async move {
            let result = Self::perform_download(
                &video_id,
                &title,
                &quality,
                &download_dir,
                download_id,
                repository.clone(),
                s3_service,
            )
            .await;

            if let Err(e) = result {
                tracing::error!(download_id = %download_id, error = %e, "Download failed");
                if let Ok(Some(mut dl)) = repository.find_by_id(&download_id).await {
                    dl.fail(e.to_string());
                    let _ = repository.update(dl).await;
                }
            }
        });

        Ok(download)
    }

    #[allow(clippy::too_many_lines)]
    async fn perform_download(
        video_id: &str,
        title: &str,
        quality: &VideoQuality,
        download_dir: &PathBuf,
        download_id: uuid::Uuid,
        repository: Arc<R>,
        s3_service: Option<Arc<S3Service>>,
    ) -> Result<(), AppError> {
        if let Ok(Some(mut dl)) = repository.find_by_id(&download_id).await {
            dl.start_fetching();
            let _ = repository.update(dl).await;
        }

        tracing::info!(video_id = %video_id, "Starting download with yt-dlp");

        let video_url = format!("https://www.youtube.com/watch?v={video_id}");

        tokio::fs::create_dir_all(download_dir)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create download dir: {e}")))?;

        let safe_title: String = title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let safe_title = safe_title.trim();

        let output_template = download_dir.join(format!("{safe_title}.%(ext)s"));

        let format_selector = match quality {
            VideoQuality::Best => "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string(),
            VideoQuality::Quality2160p => "bestvideo[height<=2160][ext=mp4]+bestaudio[ext=m4a]/best[height<=2160]".to_string(),
            VideoQuality::Quality1440p => "bestvideo[height<=1440][ext=mp4]+bestaudio[ext=m4a]/best[height<=1440]".to_string(),
            VideoQuality::Quality1080p => "bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[height<=1080]".to_string(),
            VideoQuality::Quality720p => "bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720]".to_string(),
            VideoQuality::Quality480p => "bestvideo[height<=480][ext=mp4]+bestaudio[ext=m4a]/best[height<=480]".to_string(),
            VideoQuality::Quality360p => "bestvideo[height<=360][ext=mp4]+bestaudio[ext=m4a]/best[height<=360]".to_string(),
            VideoQuality::AudioOnly => "bestaudio[ext=m4a]/bestaudio".to_string(),
        };

        if let Ok(Some(mut dl)) = repository.find_by_id(&download_id).await {
            dl.start_downloading();
            let _ = repository.update(dl).await;
        }

        let mut child = Command::new("yt-dlp")
            .args([
                "-f", &format_selector,
                "--merge-output-format", "mp4",
                "-o", output_template.to_str().unwrap_or("%(title)s.%(ext)s"),
                "--newline",
                "--progress",
                "--no-warnings",
                &video_url,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::InternalError(format!("Failed to spawn yt-dlp: {e}")))?;

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if line.contains("[download]") && line.contains('%') {
                    if let Some(percent_str) = line.split('%').next() {
                        if let Some(num_str) = percent_str.split_whitespace().last() {
                            if let Ok(progress) = num_str.parse::<f32>() {
                                if let Ok(Some(mut dl)) = repository.find_by_id(&download_id).await {
                                    dl.update_progress(progress);
                                    let _ = repository.update(dl).await;
                                }
                            }
                        }
                    }
                }

                if line.contains("[Merger]") || line.contains("[ffmpeg]") {
                    if let Ok(Some(mut dl)) = repository.find_by_id(&download_id).await {
                        dl.start_processing();
                        let _ = repository.update(dl).await;
                    }
                }
            }
        }

        let status = child.wait().await
            .map_err(|e| AppError::InternalError(format!("Failed to wait for yt-dlp: {e}")))?;

        if !status.success() {
            return Err(AppError::DownloadFailed("yt-dlp download failed".to_string()));
        }

        let extension = if *quality == VideoQuality::AudioOnly { "m4a" } else { "mp4" };
        let expected_path = download_dir.join(format!("{safe_title}.{extension}"));

        let file_path = if expected_path.exists() {
            expected_path
        } else {
            let mut found_path = None;
            if let Ok(mut entries) = tokio::fs::read_dir(download_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with(safe_title) {
                            found_path = Some(path);
                            break;
                        }
                    }
                }
            }
            found_path.ok_or_else(|| AppError::DownloadFailed("Could not find downloaded file".to_string()))?
        };

        let metadata = tokio::fs::metadata(&file_path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to get file metadata: {e}")))?;

        // Upload to S3 if configured
        let download_url = if let Some(s3) = s3_service {
            if let Ok(Some(mut dl)) = repository.find_by_id(&download_id).await {
                dl.start_processing();
                let _ = repository.update(dl).await;
            }

            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("video.mp4");
            let s3_key = format!("downloaded-videos/{}-{}", download_id, file_name);

            match s3.upload_file(&file_path, &s3_key).await {
                Ok(url) => {
                    tracing::info!(
                        video_id = %video_id,
                        download_url = %url,
                        "File uploaded to S3"
                    );
                    Some(url)
                }
                Err(e) => {
                    tracing::warn!(
                        video_id = %video_id,
                        error = %e,
                        "Failed to upload to S3, continuing without S3 URL"
                    );
                    None
                }
            }
        } else {
            None
        };

        if let Ok(Some(mut dl)) = repository.find_by_id(&download_id).await {
            dl.complete(file_path.to_string_lossy().to_string(), metadata.len(), download_url.clone());
            let _ = repository.update(dl).await;
        }

        tracing::info!(
            video_id = %video_id,
            file_path = %file_path.display(),
            size = %metadata.len(),
            download_url = ?download_url,
            "Download completed"
        );

        Ok(())
    }
}
