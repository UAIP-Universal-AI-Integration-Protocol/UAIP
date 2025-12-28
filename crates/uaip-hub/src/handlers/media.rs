//! Media Management REST API Handlers
//!
//! Endpoints for uploading, managing, and streaming media files (video, audio, images, documents).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use uaip_core::error::UaipError;
use uaip_orchestrator::media::{
    AccessLevel, MediaDimensions, MediaType, StreamProtocol, StreamQuality,
};
use uaip_orchestrator::streaming::StreamingStats;

use crate::api::rest::{ApiError, ApiResult, AppState};

/// Upload media file request
#[derive(Debug, Deserialize)]
pub struct UploadMediaRequest {
    pub filename: String,
    pub media_type: MediaType,
    pub format: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub duration_secs: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub codec_video: Option<String>,
    pub codec_audio: Option<String>,
    pub bitrate_kbps: Option<u32>,
    pub framerate_fps: Option<f32>,
    pub storage_path: String,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub tags: Vec<String>,
    pub source_device_id: Option<Uuid>,
    pub access_level: Option<AccessLevel>,
}

/// Media file response
#[derive(Debug, Serialize)]
pub struct MediaFileResponse {
    pub id: Uuid,
    pub filename: String,
    pub media_type: String,
    pub format: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub duration_secs: Option<f64>,
    pub dimensions: Option<MediaDimensions>,
    pub storage_path: String,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub uploaded_at: String,
}

/// Media list query parameters
#[derive(Debug, Deserialize)]
pub struct MediaListQuery {
    pub media_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Media list response
#[derive(Debug, Serialize)]
pub struct MediaListResponse {
    pub media_files: Vec<MediaFileResponse>,
    pub total: usize,
}

/// Create streaming session request
#[derive(Debug, Deserialize)]
pub struct CreateStreamRequest {
    pub media_id: Uuid,
    pub protocol: StreamProtocol,
    pub quality: Option<StreamQuality>,
    pub adaptive: Option<bool>,
    pub segment_duration_secs: Option<f32>,
    pub is_live: Option<bool>,
}

/// Streaming session response
#[derive(Debug, Serialize)]
pub struct StreamSessionResponse {
    pub id: Uuid,
    pub media_id: Uuid,
    pub protocol: String,
    pub quality: String,
    pub state: String,
    pub clients_count: usize,
    pub started_at: String,
    pub stream_url: Option<String>,
    pub stats: StreamingStats,
}

/// Upload a media file
pub async fn upload_media(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UploadMediaRequest>,
) -> ApiResult<Json<MediaFileResponse>> {
    info!("Uploading media file: {}", request.filename);

    let media_id = Uuid::new_v4();
    let access_level = request.access_level.unwrap_or(AccessLevel::Private);

    // Store in database if available
    if let Some(pool) = &state.db_pool {

        match sqlx::query(
            r#"
            INSERT INTO media_files (
                id, filename, media_type, format, mime_type, size_bytes,
                duration_secs, width, height, codec_video, codec_audio,
                bitrate_kbps, framerate_fps, storage_path, url, thumbnail_url,
                tags, status, source_device_id, access_level
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            "#,
        )
        .bind(media_id)
        .bind(&request.filename)
        .bind(format!("{:?}", request.media_type).to_lowercase())
        .bind(&request.format)
        .bind(&request.mime_type)
        .bind(request.size_bytes as i64)
        .bind(request.duration_secs)
        .bind(request.width.map(|w| w as i32))
        .bind(request.height.map(|h| h as i32))
        .bind(&request.codec_video)
        .bind(&request.codec_audio)
        .bind(request.bitrate_kbps.map(|b| b as i32))
        .bind(request.framerate_fps)
        .bind(&request.storage_path)
        .bind(&request.url)
        .bind(&request.thumbnail_url)
        .bind(&request.tags)
        .bind("pending")
        .bind(request.source_device_id)
        .bind(format!("{:?}", access_level).to_lowercase())
        .execute(pool)
        .await
        {
            Ok(_) => {
                info!("Stored media file {} in database", media_id);
            }
            Err(e) => {
                error!("Failed to store media file in database: {}", e);
                return Err(ApiError(UaipError::DatabaseError(format!(
                    "Failed to store media: {}",
                    e
                ))));
            }
        }
    }

    let dimensions = if request.width.is_some() && request.height.is_some() {
        Some(MediaDimensions {
            width: request.width.unwrap(),
            height: request.height.unwrap(),
        })
    } else {
        None
    };

    Ok(Json(MediaFileResponse {
        id: media_id,
        filename: request.filename,
        media_type: format!("{:?}", request.media_type),
        format: request.format,
        mime_type: request.mime_type,
        size_bytes: request.size_bytes,
        duration_secs: request.duration_secs,
        dimensions,
        storage_path: request.storage_path,
        url: request.url,
        thumbnail_url: request.thumbnail_url,
        tags: request.tags,
        status: "pending".to_string(),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// List media files
pub async fn list_media(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MediaListQuery>,
) -> ApiResult<Json<MediaListResponse>> {
    info!("Listing media files");

    let mut media_files = Vec::new();

    if let Some(pool) = &state.db_pool {
        let limit = query.limit.unwrap_or(50).min(100);
        let offset = query.offset.unwrap_or(0);

        let mut sql = String::from(
            "SELECT id, filename, media_type, format, mime_type, size_bytes,
             duration_secs, width, height, storage_path, url, thumbnail_url,
             tags, status, uploaded_at
             FROM media_files WHERE 1=1",
        );

        if let Some(ref media_type) = query.media_type {
            sql.push_str(&format!(" AND media_type = '{}'", media_type));
        }

        if let Some(ref status) = query.status {
            sql.push_str(&format!(" AND status = '{}'", status));
        }

        sql.push_str(" ORDER BY uploaded_at DESC");
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        match sqlx::query(&sql).fetch_all(pool).await {
            Ok(records) => {
                for record in records {
                    let id: Uuid = record.try_get("id").unwrap_or_default();
                    let filename: String = record.try_get("filename").unwrap_or_default();
                    let media_type: String = record.try_get("media_type").unwrap_or_default();
                    let format: String = record.try_get("format").unwrap_or_default();
                    let mime_type: String = record.try_get("mime_type").unwrap_or_default();
                    let size_bytes: i64 = record.try_get("size_bytes").unwrap_or_default();
                    let duration_secs: Option<f64> = record.try_get("duration_secs").ok();
                    let width: Option<i32> = record.try_get("width").ok();
                    let height: Option<i32> = record.try_get("height").ok();
                    let storage_path: String = record.try_get("storage_path").unwrap_or_default();
                    let url: Option<String> = record.try_get("url").ok();
                    let thumbnail_url: Option<String> = record.try_get("thumbnail_url").ok();
                    let tags: Vec<String> = record.try_get("tags").unwrap_or_default();
                    let status: String = record.try_get("status").unwrap_or_default();
                    let uploaded_at: chrono::NaiveDateTime =
                        record.try_get("uploaded_at").unwrap_or_default();

                    let dimensions = if let (Some(w), Some(h)) = (width, height) {
                        Some(MediaDimensions {
                            width: w as u32,
                            height: h as u32,
                        })
                    } else {
                        None
                    };

                    media_files.push(MediaFileResponse {
                        id,
                        filename,
                        media_type,
                        format,
                        mime_type,
                        size_bytes: size_bytes as u64,
                        duration_secs,
                        dimensions,
                        storage_path,
                        url,
                        thumbnail_url,
                        tags,
                        status,
                        uploaded_at: uploaded_at.and_utc().to_rfc3339(),
                    });
                }
            }
            Err(e) => {
                error!("Failed to fetch media files from database: {}", e);
            }
        }
    }

    let total = media_files.len();
    Ok(Json(MediaListResponse { media_files, total }))
}

/// Get media file by ID
pub async fn get_media(
    State(state): State<Arc<AppState>>,
    Path(media_id): Path<Uuid>,
) -> ApiResult<Json<MediaFileResponse>> {
    info!("Getting media file: {}", media_id);

    if let Some(pool) = &state.db_pool {
        match sqlx::query(
            r#"
            SELECT id, filename, media_type, format, mime_type, size_bytes,
                   duration_secs, width, height, storage_path, url, thumbnail_url,
                   tags, status, uploaded_at
            FROM media_files
            WHERE id = $1
            "#,
        )
        .bind(media_id)
        .fetch_one(pool)
        .await
        {
            Ok(record) => {
                let id: Uuid = record.try_get("id").unwrap_or_default();
                let filename: String = record.try_get("filename").unwrap_or_default();
                let media_type: String = record.try_get("media_type").unwrap_or_default();
                let format: String = record.try_get("format").unwrap_or_default();
                let mime_type: String = record.try_get("mime_type").unwrap_or_default();
                let size_bytes: i64 = record.try_get("size_bytes").unwrap_or_default();
                let duration_secs: Option<f64> = record.try_get("duration_secs").ok();
                let width: Option<i32> = record.try_get("width").ok();
                let height: Option<i32> = record.try_get("height").ok();
                let storage_path: String = record.try_get("storage_path").unwrap_or_default();
                let url: Option<String> = record.try_get("url").ok();
                let thumbnail_url: Option<String> = record.try_get("thumbnail_url").ok();
                let tags: Vec<String> = record.try_get("tags").unwrap_or_default();
                let status: String = record.try_get("status").unwrap_or_default();
                let uploaded_at: chrono::NaiveDateTime =
                    record.try_get("uploaded_at").unwrap_or_default();

                let dimensions = if let (Some(w), Some(h)) = (width, height) {
                    Some(MediaDimensions {
                        width: w as u32,
                        height: h as u32,
                    })
                } else {
                    None
                };

                return Ok(Json(MediaFileResponse {
                    id,
                    filename,
                    media_type,
                    format,
                    mime_type,
                    size_bytes: size_bytes as u64,
                    duration_secs,
                    dimensions,
                    storage_path,
                    url,
                    thumbnail_url,
                    tags,
                    status,
                    uploaded_at: uploaded_at.and_utc().to_rfc3339(),
                }));
            }
            Err(e) => {
                error!("Failed to fetch media file from database: {}", e);
                return Err(ApiError(UaipError::NotFound(format!(
                    "Media file {} not found",
                    media_id
                ))));
            }
        }
    }

    Err(ApiError(UaipError::NotFound(format!(
        "Media file {} not found",
        media_id
    ))))
}

/// Delete media file
pub async fn delete_media(
    State(state): State<Arc<AppState>>,
    Path(media_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    info!("Deleting media file: {}", media_id);

    if let Some(pool) = &state.db_pool {
        match sqlx::query("DELETE FROM media_files WHERE id = $1")
            .bind(media_id)
            .execute(pool)
            .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    info!("Deleted media file {}", media_id);
                    return Ok(StatusCode::NO_CONTENT);
                } else {
                    return Err(ApiError(UaipError::NotFound(format!(
                        "Media file {} not found",
                        media_id
                    ))));
                }
            }
            Err(e) => {
                error!("Failed to delete media file: {}", e);
                return Err(ApiError(UaipError::DatabaseError(format!(
                    "Failed to delete media: {}",
                    e
                ))));
            }
        }
    }

    Err(ApiError(UaipError::NotFound(format!(
        "Media file {} not found",
        media_id
    ))))
}

/// Create streaming session
pub async fn create_stream_session(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateStreamRequest>,
) -> ApiResult<Json<StreamSessionResponse>> {
    info!(
        "Creating streaming session for media: {}",
        request.media_id
    );

    let session_id = Uuid::new_v4();
    let quality = request.quality.unwrap_or(StreamQuality::Auto);
    let adaptive = request.adaptive.unwrap_or(true);
    let segment_duration = request.segment_duration_secs.unwrap_or(6.0);
    let is_live = request.is_live.unwrap_or(false);

    // Store in database if available
    if let Some(pool) = &state.db_pool {
        match sqlx::query(
            r#"
            INSERT INTO stream_configs (
                id, media_id, protocol, quality, adaptive,
                segment_duration_secs, is_live
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(session_id)
        .bind(request.media_id)
        .bind(format!("{:?}", request.protocol))
        .bind(format!("{:?}", quality))
        .bind(adaptive)
        .bind(segment_duration)
        .bind(is_live)
        .execute(pool)
        .await
        {
            Ok(_) => {
                info!("Created stream session {} in database", session_id);
            }
            Err(e) => {
                error!("Failed to create stream session in database: {}", e);
                return Err(ApiError(UaipError::DatabaseError(format!(
                    "Failed to create stream: {}",
                    e
                ))));
            }
        }
    }

    Ok(Json(StreamSessionResponse {
        id: session_id,
        media_id: request.media_id,
        protocol: format!("{:?}", request.protocol),
        quality: format!("{:?}", quality),
        state: "Initializing".to_string(),
        clients_count: 0,
        started_at: chrono::Utc::now().to_rfc3339(),
        stream_url: None,
        stats: StreamingStats::default(),
    }))
}

/// Get streaming session
pub async fn get_stream_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<StreamSessionResponse>> {
    info!("Getting streaming session: {}", session_id);

    if let Some(pool) = &state.db_pool {
        match sqlx::query(
            r#"
            SELECT id, media_id, protocol, quality, stream_url, created_at
            FROM stream_configs
            WHERE id = $1 AND active = TRUE
            "#,
        )
        .bind(session_id)
        .fetch_one(pool)
        .await
        {
            Ok(record) => {
                let id: Uuid = record.try_get("id").unwrap_or_default();
                let media_id: Uuid = record.try_get("media_id").unwrap_or_default();
                let protocol: String = record.try_get("protocol").unwrap_or_default();
                let quality: String = record.try_get("quality").unwrap_or_default();
                let stream_url: Option<String> = record.try_get("stream_url").ok();
                let created_at: chrono::NaiveDateTime =
                    record.try_get("created_at").unwrap_or_default();

                return Ok(Json(StreamSessionResponse {
                    id,
                    media_id,
                    protocol,
                    quality,
                    state: "Streaming".to_string(),
                    clients_count: 0,
                    started_at: created_at.and_utc().to_rfc3339(),
                    stream_url,
                    stats: StreamingStats::default(),
                }));
            }
            Err(e) => {
                error!("Failed to fetch stream session from database: {}", e);
                return Err(ApiError(UaipError::NotFound(format!(
                    "Stream session {} not found",
                    session_id
                ))));
            }
        }
    }

    Err(ApiError(UaipError::NotFound(format!(
        "Stream session {} not found",
        session_id
    ))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_request_deserialize() {
        let json = r#"{
            "filename": "test.mp4",
            "media_type": "Video",
            "format": "mp4",
            "mime_type": "video/mp4",
            "size_bytes": 1024000,
            "storage_path": "/media/test.mp4",
            "tags": ["test", "video"]
        }"#;

        let request: Result<UploadMediaRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
    }
}
