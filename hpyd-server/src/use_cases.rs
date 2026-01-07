use std::path::PathBuf;
use std::sync::Arc;

use hpyd_downloads::{
    CancelDownload, CreateDownload, GetDownload, GetVideoInfo, InMemoryDownloadRepository,
    ListDownloads,
};

use crate::config::Config;

pub struct UseCases {
    pub get_video_info: Arc<GetVideoInfo>,
    pub create_download: Arc<CreateDownload<InMemoryDownloadRepository>>,
    pub get_download: Arc<GetDownload<InMemoryDownloadRepository>>,
    pub list_downloads: Arc<ListDownloads<InMemoryDownloadRepository>>,
    pub cancel_download: Arc<CancelDownload<InMemoryDownloadRepository>>,
}

pub fn init_use_cases(config: &Config) -> UseCases {
    let download_repository = InMemoryDownloadRepository::new();
    let download_dir = PathBuf::from(&config.download_dir);

    UseCases {
        get_video_info: Arc::new(GetVideoInfo),
        create_download: Arc::new(CreateDownload::new(download_repository.clone(), download_dir)),
        get_download: Arc::new(GetDownload::new(download_repository.clone())),
        list_downloads: Arc::new(ListDownloads::new(download_repository.clone())),
        cancel_download: Arc::new(CancelDownload::new(download_repository)),
    }
}
