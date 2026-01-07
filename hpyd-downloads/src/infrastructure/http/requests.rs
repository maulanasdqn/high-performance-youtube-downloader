use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::VideoQuality;

#[derive(Debug, Deserialize, ToSchema)]
pub struct VideoInfoRequest {
    #[schema(example = "https://www.youtube.com/watch?v=dQw4w9WgXcQ")]
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDownloadRequest {
    #[schema(example = "https://www.youtube.com/watch?v=dQw4w9WgXcQ")]
    pub url: String,

    #[serde(default)]
    pub quality: VideoQuality,
}
