use std::path::PathBuf;
use std::sync::Arc;

use hpyd_downloads::{
    CancelDownload, CreateDownload, GetDownload, GetVideoInfo, InMemoryDownloadRepository,
    ListDownloads, S3Config, S3Service,
};

use crate::config::Config;

pub struct UseCases {
    pub get_video_info: Arc<GetVideoInfo>,
    pub create_download: Arc<CreateDownload<InMemoryDownloadRepository>>,
    pub get_download: Arc<GetDownload<InMemoryDownloadRepository>>,
    pub list_downloads: Arc<ListDownloads<InMemoryDownloadRepository>>,
    pub cancel_download: Arc<CancelDownload<InMemoryDownloadRepository>>,
}

pub async fn init_use_cases(config: &Config) -> UseCases {
    let download_repository = InMemoryDownloadRepository::new();
    let download_dir = PathBuf::from(&config.download_dir);

    // Initialize S3 service if configured
    let s3_service = if let Some(s3_config) = S3Config::from_env() {
        tracing::info!(
            endpoint = %s3_config.endpoint,
            bucket = %s3_config.bucket,
            "S3 storage configured"
        );
        Some(Arc::new(S3Service::new(&s3_config).await))
    } else {
        tracing::warn!("S3 not configured, downloads will be stored locally only");
        None
    };

    UseCases {
        get_video_info: Arc::new(GetVideoInfo),
        create_download: Arc::new(CreateDownload::new(download_repository.clone(), download_dir, s3_service)),
        get_download: Arc::new(GetDownload::new(download_repository.clone())),
        list_downloads: Arc::new(ListDownloads::new(download_repository.clone())),
        cancel_download: Arc::new(CancelDownload::new(download_repository)),
    }
}
