use axum::{routing::{get, post}, Router};

use super::handlers;

pub fn download_routes() -> Router {
    Router::new()
        .route("/video-info", post(handlers::get_video_info))
        .route(
            "/downloads",
            post(handlers::create_download::<crate::infrastructure::InMemoryDownloadRepository>)
                .get(handlers::list_downloads::<crate::infrastructure::InMemoryDownloadRepository>),
        )
        .route(
            "/downloads/{id}",
            get(handlers::get_download::<crate::infrastructure::InMemoryDownloadRepository>),
        )
        .route(
            "/downloads/{id}/cancel",
            post(handlers::cancel_download::<crate::infrastructure::InMemoryDownloadRepository>),
        )
}

