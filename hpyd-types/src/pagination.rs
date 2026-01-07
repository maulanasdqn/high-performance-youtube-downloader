use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub const DEFAULT_PAGE: u32 = 1;
pub const DEFAULT_PER_PAGE: u32 = 20;
pub const MAX_PER_PAGE: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct PaginationQuery {
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub per_page: Option<u32>,
}

impl PaginationQuery {
    #[must_use]
    pub fn page(&self) -> u32 {
        self.page.map_or(DEFAULT_PAGE, |p| p.max(1))
    }

    #[must_use]
    pub fn per_page(&self) -> u32 {
        self.per_page
            .map_or(DEFAULT_PER_PAGE, |p| p.clamp(1, MAX_PER_PAGE))
    }

    #[must_use]
    pub fn offset(&self) -> u32 {
        (self.page() - 1) * self.per_page()
    }
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(DEFAULT_PAGE),
            per_page: Some(DEFAULT_PER_PAGE),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub total_items: u64,
}

impl PaginationMeta {
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]
    pub fn new(page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / f64::from(per_page)).ceil() as u32;
        Self {
            page,
            per_page,
            total_pages,
            total_items,
        }
    }
}
