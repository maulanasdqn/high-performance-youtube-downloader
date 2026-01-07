pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::{
    CreateDownload, GetDownload, GetVideoInfo, ListDownloads, CancelDownload,
};
pub use domain::{Download, DownloadRepository, DownloadStatus, VideoInfo, VideoQuality};
pub use infrastructure::{
    download_routes, CreateDownloadRequest, DownloadResponse, InMemoryDownloadRepository,
    VideoInfoRequest, VideoInfoResponse,
};

