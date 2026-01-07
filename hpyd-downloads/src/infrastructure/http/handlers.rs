use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use hpyd_errors::AppError;
use hpyd_types::{ListResponse, PaginationQuery, SingleResponse};
use uuid::Uuid;

use crate::application::{CancelDownload, CreateDownload, GetDownload, GetVideoInfo, ListDownloads};
use crate::domain::DownloadRepository;
use crate::infrastructure::http::requests::{CreateDownloadRequest, VideoInfoRequest};
use crate::infrastructure::http::responses::{DownloadResponse, VideoInfoResponse};

#[utoipa::path(
    post,
    path = "/api/v1/video-info",
    tag = "Downloads",
    request_body = VideoInfoRequest,
    responses(
        (status = 200, description = "Video information retrieved", body = SingleResponse<VideoInfoResponse>),
        (status = 400, description = "Invalid URL"),
        (status = 422, description = "Video unavailable")
    )
)]
pub async fn get_video_info(
    Extension(use_case): Extension<Arc<GetVideoInfo>>,
    Json(payload): Json<VideoInfoRequest>,
) -> Result<Json<SingleResponse<VideoInfoResponse>>, AppError> {
    let video_info = GetVideoInfo::execute(use_case, &payload.url).await?;
    Ok(Json(SingleResponse::new(VideoInfoResponse::from(video_info))))
}

#[utoipa::path(
    post,
    path = "/api/v1/downloads",
    tag = "Downloads",
    request_body = CreateDownloadRequest,
    responses(
        (status = 201, description = "Download created", body = SingleResponse<DownloadResponse>),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn create_download<R: DownloadRepository + 'static>(
    Extension(use_case): Extension<Arc<CreateDownload<R>>>,
    Extension(get_video_info): Extension<Arc<GetVideoInfo>>,
    Json(payload): Json<CreateDownloadRequest>,
) -> Result<(StatusCode, Json<SingleResponse<DownloadResponse>>), AppError> {
    let video_info = GetVideoInfo::execute(get_video_info, &payload.url).await?;

    let download = use_case
        .execute(
            video_info.video_id,
            payload.url,
            video_info.title,
            payload.quality,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(SingleResponse::with_message(
            "Download queued successfully",
            DownloadResponse::from(download),
        )),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/downloads/{id}",
    tag = "Downloads",
    params(
        ("id" = Uuid, Path, description = "Download ID")
    ),
    responses(
        (status = 200, description = "Download found", body = SingleResponse<DownloadResponse>),
        (status = 404, description = "Download not found")
    )
)]
pub async fn get_download<R: DownloadRepository + 'static>(
    Extension(use_case): Extension<Arc<GetDownload<R>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SingleResponse<DownloadResponse>>, AppError> {
    let download = use_case.execute(id).await?;
    Ok(Json(SingleResponse::new(DownloadResponse::from(download))))
}

#[utoipa::path(
    get,
    path = "/api/v1/downloads",
    tag = "Downloads",
    params(PaginationQuery),
    responses(
        (status = 200, description = "Downloads list", body = ListResponse<DownloadResponse>)
    )
)]
pub async fn list_downloads<R: DownloadRepository + 'static>(
    Extension(use_case): Extension<Arc<ListDownloads<R>>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListResponse<DownloadResponse>>, AppError> {
    let (downloads, meta) = use_case.execute(pagination.page(), pagination.per_page()).await?;

    let responses: Vec<DownloadResponse> = downloads.into_iter().map(DownloadResponse::from).collect();

    Ok(Json(ListResponse::new(responses, meta)))
}

#[utoipa::path(
    post,
    path = "/api/v1/downloads/{id}/cancel",
    tag = "Downloads",
    params(
        ("id" = Uuid, Path, description = "Download ID")
    ),
    responses(
        (status = 200, description = "Download cancelled", body = SingleResponse<DownloadResponse>),
        (status = 400, description = "Cannot cancel download"),
        (status = 404, description = "Download not found")
    )
)]
pub async fn cancel_download<R: DownloadRepository + 'static>(
    Extension(use_case): Extension<Arc<CancelDownload<R>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SingleResponse<DownloadResponse>>, AppError> {
    let download = use_case.execute(id).await?;
    Ok(Json(SingleResponse::with_message(
        "Download cancelled",
        DownloadResponse::from(download),
    )))
}
