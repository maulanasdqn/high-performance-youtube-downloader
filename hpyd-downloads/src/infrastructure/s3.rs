use std::path::Path;

use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::{config::Region, Client};
use hpyd_errors::AppError;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub public_url: String,
}

impl S3Config {
    pub fn from_env() -> Option<Self> {
        let endpoint = std::env::var("S3_ENDPOINT").ok()?;
        let bucket = std::env::var("S3_BUCKET").ok()?;
        let access_key = std::env::var("S3_ACCESS_KEY").ok()?;
        let secret_key = std::env::var("S3_SECRET_KEY").ok()?;
        let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let public_url = std::env::var("S3_PUBLIC_URL").ok()?;

        Some(Self {
            endpoint,
            bucket,
            access_key,
            secret_key,
            region,
            public_url,
        })
    }
}

#[derive(Debug, Clone)]
pub struct S3Service {
    client: Client,
    bucket: String,
    public_url: String,
}

impl S3Service {
    pub async fn new(config: &S3Config) -> Self {
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "hpyd",
        );

        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(Region::new(config.region.clone()))
            .endpoint_url(&config.endpoint)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket: config.bucket.clone(),
            public_url: config.public_url.clone(),
        }
    }

    pub async fn upload_file(&self, file_path: &Path, key: &str) -> Result<String, AppError> {
        let mut file = File::open(file_path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to open file for upload: {e}")))?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to read file for upload: {e}")))?;

        let content_type = if key.ends_with(".mp4") {
            "video/mp4"
        } else if key.ends_with(".m4a") {
            "audio/mp4"
        } else {
            "application/octet-stream"
        };

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(contents.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to upload to S3: {e}")))?;

        // public_url already contains the bucket path, so just append the key
        let download_url = format!("{}/{}", self.public_url, key);

        tracing::info!(
            key = %key,
            download_url = %download_url,
            "File uploaded to S3"
        );

        Ok(download_url)
    }
}
