use axum::{response::Redirect, routing::get, Extension, Router};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use hpyd_downloads::download_routes;

use crate::api_doc::ApiDoc;
use crate::config::Config;
use crate::health;
use crate::use_cases::UseCases;

async fn redirect_to_docs() -> Redirect {
    Redirect::permanent("/docs")
}

pub fn build_router(use_cases: UseCases, _config: &Config) -> Router {
    let api_v1 = Router::new()
        .merge(download_routes())
        .layer(Extension(use_cases.get_video_info))
        .layer(Extension(use_cases.create_download))
        .layer(Extension(use_cases.get_download))
        .layer(Extension(use_cases.list_downloads))
        .layer(Extension(use_cases.cancel_download));

    Router::new()
        .route("/", get(redirect_to_docs))
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_v1)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(true)
                        .level(tracing::Level::INFO),
                )
                .on_response(
                    DefaultOnResponse::new()
                        .include_headers(true)
                        .latency_unit(LatencyUnit::Millis)
                        .level(tracing::Level::INFO),
                ),
        )
        .layer(CorsLayer::permissive())
}

