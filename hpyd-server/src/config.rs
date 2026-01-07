#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub download_dir: String,
    #[allow(dead_code)]
    pub max_concurrent_downloads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            download_dir: "./downloads".to_string(),
            max_concurrent_downloads: 3,
        }
    }
}

pub fn load_environment_config() -> Config {
    dotenvy::dotenv().ok();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let download_dir =
        std::env::var("DOWNLOAD_DIR").unwrap_or_else(|_| "./downloads".to_string());

    let max_concurrent_downloads: usize = std::env::var("MAX_CONCURRENT_DOWNLOADS")
        .unwrap_or_else(|_| "3".to_string())
        .parse()
        .unwrap_or(3);

    Config {
        host,
        port,
        download_dir,
        max_concurrent_downloads,
    }
}
