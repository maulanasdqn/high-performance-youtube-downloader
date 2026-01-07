#![allow(clippy::option_if_let_else)]

use crate::pagination::PaginationMeta;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const API_VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
    pub version: String,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            version: API_VERSION.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SingleResponse<T> {
    pub message: String,
    pub data: T,
    pub version: String,
}

impl<T> SingleResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            message: "Success".to_string(),
            data,
            version: API_VERSION.to_string(),
        }
    }

    pub fn with_message(message: impl Into<String>, data: T) -> Self {
        Self {
            message: message.into(),
            data,
            version: API_VERSION.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageOnlyResponse {
    pub message: String,
    pub version: String,
}

impl MessageOnlyResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            version: API_VERSION.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListResponse<T> {
    pub message: String,
    pub data: Vec<T>,
    pub meta: PaginationMeta,
    pub version: String,
}

impl<T> ListResponse<T> {
    pub fn new(data: Vec<T>, meta: PaginationMeta) -> Self {
        Self {
            message: "Success".to_string(),
            data,
            meta,
            version: API_VERSION.to_string(),
        }
    }

    pub fn with_message(message: impl Into<String>, data: Vec<T>, meta: PaginationMeta) -> Self {
        Self {
            message: message.into(),
            data,
            meta,
            version: API_VERSION.to_string(),
        }
    }
}
