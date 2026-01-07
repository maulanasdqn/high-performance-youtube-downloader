pub mod http;
pub mod persistence;

pub use http::{
    download_routes, CreateDownloadRequest, DownloadResponse, VideoInfoRequest, VideoInfoResponse,
};
pub use persistence::InMemoryDownloadRepository;

