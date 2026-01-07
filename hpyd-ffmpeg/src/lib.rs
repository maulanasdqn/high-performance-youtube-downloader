use std::path::Path;
use std::process::Stdio;

use anyhow::{anyhow, Result};
use tokio::process::Command;

pub async fn check_ffmpeg() -> Result<bool> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    Ok(output.is_ok())
}

pub async fn merge_streams(
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
) -> Result<()> {
    tracing::info!(
        video = %video_path.display(),
        audio = %audio_path.display(),
        output = %output_path.display(),
        "Merging video and audio streams"
    );

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().ok_or_else(|| anyhow!("Invalid video path"))?,
            "-i",
            audio_path.to_str().ok_or_else(|| anyhow!("Invalid audio path"))?,
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-y",
            output_path.to_str().ok_or_else(|| anyhow!("Invalid output path"))?,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    if status.success() {
        tracing::info!("Successfully merged streams");
        Ok(())
    } else {
        Err(anyhow!("FFmpeg merge failed with exit code: {:?}", status.code()))
    }
}

pub async fn extract_audio(video_path: &Path, output_path: &Path) -> Result<()> {
    tracing::info!(
        video = %video_path.display(),
        output = %output_path.display(),
        "Extracting audio from video"
    );

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().ok_or_else(|| anyhow!("Invalid video path"))?,
            "-vn",
            "-acodec",
            "copy",
            "-y",
            output_path.to_str().ok_or_else(|| anyhow!("Invalid output path"))?,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    if status.success() {
        tracing::info!("Successfully extracted audio");
        Ok(())
    } else {
        Err(anyhow!("FFmpeg extract failed with exit code: {:?}", status.code()))
    }
}
