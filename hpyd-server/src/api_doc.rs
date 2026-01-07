#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use hpyd_downloads::{
    CreateDownloadRequest, Download, DownloadResponse, DownloadStatus, VideoInfoRequest,
    VideoInfoResponse, VideoQuality,
};
use hpyd_types::{ErrorResponse, ListResponse, PaginationMeta, PaginationQuery, SingleResponse};

use crate::health::{HealthResponse, ReadinessChecks, ReadinessResponse};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "High Performance YouTube Downloader API",
        version = "0.1.0",
        description = "REST API for downloading YouTube videos with high performance",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Downloads", description = "Video download management endpoints")
    ),
    paths(
        crate::health::health_check,
        crate::health::readiness_check,
        hpyd_downloads::infrastructure::http::handlers::get_video_info,
        hpyd_downloads::infrastructure::http::handlers::create_download,
        hpyd_downloads::infrastructure::http::handlers::get_download,
        hpyd_downloads::infrastructure::http::handlers::list_downloads,
        hpyd_downloads::infrastructure::http::handlers::cancel_download,
    ),
    components(schemas(
        HealthResponse,
        ReadinessResponse,
        ReadinessChecks,
        ErrorResponse,
        PaginationMeta,
        PaginationQuery,
        Download,
        DownloadStatus,
        DownloadResponse,
        VideoQuality,
        VideoInfoRequest,
        VideoInfoResponse,
        CreateDownloadRequest,
        SingleResponse<DownloadResponse>,
        SingleResponse<VideoInfoResponse>,
        ListResponse<DownloadResponse>,
    ))
)]
pub struct ApiDoc;
