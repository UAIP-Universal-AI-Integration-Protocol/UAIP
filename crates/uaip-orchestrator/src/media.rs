//! Media Management Module
//!
//! Handles video, audio, image, and document processing, storage, and streaming.
//! Supports multiple formats, compression, transcoding, and real-time streaming.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Media file metadata and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    /// Unique media file ID
    pub id: Uuid,

    /// Original filename
    pub filename: String,

    /// Media type
    pub media_type: MediaType,

    /// File format/extension
    pub format: String,

    /// MIME type
    pub mime_type: String,

    /// File size in bytes
    pub size_bytes: u64,

    /// Duration in seconds (for audio/video)
    pub duration_secs: Option<f64>,

    /// Dimensions (for images/video)
    pub dimensions: Option<MediaDimensions>,

    /// Codec information
    pub codec: Option<CodecInfo>,

    /// Bitrate in kbps
    pub bitrate_kbps: Option<u32>,

    /// Frame rate (for video)
    pub framerate_fps: Option<f32>,

    /// Storage location
    pub storage_path: String,

    /// URL for access
    pub url: Option<String>,

    /// Thumbnail URL (for video/image)
    pub thumbnail_url: Option<String>,

    /// Metadata tags
    pub tags: Vec<String>,

    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,

    /// Processing status
    pub status: MediaStatus,

    /// Source device ID
    pub source_device: Option<String>,

    /// Access control
    pub access_level: AccessLevel,
}

impl MediaFile {
    /// Create a new media file record
    pub fn new(filename: String, media_type: MediaType, format: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            filename,
            media_type,
            format: format.clone(),
            mime_type: Self::detect_mime_type(&media_type, &format),
            size_bytes: 0,
            duration_secs: None,
            dimensions: None,
            codec: None,
            bitrate_kbps: None,
            framerate_fps: None,
            storage_path: String::new(),
            url: None,
            thumbnail_url: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            uploaded_at: Utc::now(),
            status: MediaStatus::Pending,
            source_device: None,
            access_level: AccessLevel::Private,
        }
    }

    /// Detect MIME type from media type and format
    fn detect_mime_type(media_type: &MediaType, format: &str) -> String {
        match media_type {
            MediaType::Video => match format.to_lowercase().as_str() {
                "mp4" => "video/mp4",
                "webm" => "video/webm",
                "mkv" => "video/x-matroska",
                "avi" => "video/x-msvideo",
                "mov" => "video/quicktime",
                "flv" => "video/x-flv",
                _ => "video/mp4",
            },
            MediaType::Audio => match format.to_lowercase().as_str() {
                "mp3" => "audio/mpeg",
                "wav" => "audio/wav",
                "ogg" => "audio/ogg",
                "flac" => "audio/flac",
                "aac" => "audio/aac",
                "m4a" => "audio/mp4",
                _ => "audio/mpeg",
            },
            MediaType::Image => match format.to_lowercase().as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "webp" => "image/webp",
                "svg" => "image/svg+xml",
                "bmp" => "image/bmp",
                _ => "image/jpeg",
            },
            MediaType::Document => match format.to_lowercase().as_str() {
                "pdf" => "application/pdf",
                "doc" | "docx" => "application/msword",
                "txt" => "text/plain",
                "json" => "application/json",
                "xml" => "application/xml",
                _ => "application/octet-stream",
            },
        }
        .to_string()
    }

    /// Check if media is ready for playback
    pub fn is_ready(&self) -> bool {
        matches!(self.status, MediaStatus::Ready)
    }

    /// Check if media requires processing
    pub fn needs_processing(&self) -> bool {
        matches!(self.status, MediaStatus::Pending | MediaStatus::Processing)
    }
}

/// Type of media
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// Video files
    Video,
    /// Audio files
    Audio,
    /// Image files
    Image,
    /// Document files (PDF, text, etc.)
    Document,
}

/// Media dimensions (width x height)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MediaDimensions {
    pub width: u32,
    pub height: u32,
}

impl MediaDimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Check if HD (720p or higher)
    pub fn is_hd(&self) -> bool {
        self.height >= 720
    }

    /// Check if Full HD (1080p)
    pub fn is_full_hd(&self) -> bool {
        self.height >= 1080
    }

    /// Check if 4K
    pub fn is_4k(&self) -> bool {
        self.height >= 2160
    }
}

/// Codec information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecInfo {
    /// Video codec (e.g., H.264, VP9, AV1)
    pub video: Option<String>,
    /// Audio codec (e.g., AAC, Opus, MP3)
    pub audio: Option<String>,
    /// Codec profile
    pub profile: Option<String>,
}

/// Media processing status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaStatus {
    /// Pending processing
    Pending,
    /// Currently being processed
    Processing,
    /// Ready for use
    Ready,
    /// Processing failed
    Failed,
    /// Archived/inactive
    Archived,
}

/// Access control level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    /// Only owner can access
    Private,
    /// Specific users/agents can access
    Restricted,
    /// All authenticated users can access
    Internal,
    /// Publicly accessible
    Public,
}

/// Media streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Stream ID
    pub id: Uuid,

    /// Source media file ID
    pub media_id: Uuid,

    /// Streaming protocol
    pub protocol: StreamProtocol,

    /// Target quality/bitrate
    pub quality: StreamQuality,

    /// Adaptive bitrate streaming enabled
    pub adaptive: bool,

    /// Available quality levels (for ABR)
    pub quality_levels: Vec<StreamQuality>,

    /// Segment duration in seconds (for HLS/DASH)
    pub segment_duration_secs: f32,

    /// Stream URL
    pub stream_url: String,

    /// Live stream vs VOD
    pub is_live: bool,

    /// Buffer size in seconds
    pub buffer_secs: f32,
}

impl StreamConfig {
    /// Create a new stream configuration
    pub fn new(media_id: Uuid, protocol: StreamProtocol) -> Self {
        Self {
            id: Uuid::new_v4(),
            media_id,
            protocol,
            quality: StreamQuality::Auto,
            adaptive: true,
            quality_levels: vec![
                StreamQuality::Low,
                StreamQuality::Medium,
                StreamQuality::High,
            ],
            segment_duration_secs: 6.0,
            stream_url: String::new(),
            is_live: false,
            buffer_secs: 30.0,
        }
    }

    /// Create configuration for live streaming
    pub fn live(media_id: Uuid) -> Self {
        let mut config = Self::new(media_id, StreamProtocol::WebRtc);
        config.is_live = true;
        config.buffer_secs = 2.0;
        config.segment_duration_secs = 2.0;
        config
    }
}

/// Streaming protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StreamProtocol {
    /// HTTP Live Streaming (Apple)
    Hls,
    /// Dynamic Adaptive Streaming over HTTP (MPEG-DASH)
    Dash,
    /// WebRTC for real-time streaming
    WebRtc,
    /// Real-Time Messaging Protocol
    Rtmp,
    /// Progressive download
    Http,
}

/// Stream quality preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamQuality {
    /// Auto-select based on bandwidth
    Auto,
    /// 240p (low quality, ~400 kbps)
    Low,
    /// 480p (medium quality, ~1 Mbps)
    Medium,
    /// 720p (HD quality, ~2.5 Mbps)
    High,
    /// 1080p (Full HD, ~5 Mbps)
    FullHd,
    /// 4K (Ultra HD, ~15 Mbps)
    UltraHd,
}

impl StreamQuality {
    /// Get target bitrate in kbps
    pub fn bitrate_kbps(&self) -> u32 {
        match self {
            StreamQuality::Auto => 2500,
            StreamQuality::Low => 400,
            StreamQuality::Medium => 1000,
            StreamQuality::High => 2500,
            StreamQuality::FullHd => 5000,
            StreamQuality::UltraHd => 15000,
        }
    }

    /// Get resolution height
    pub fn height(&self) -> Option<u32> {
        match self {
            StreamQuality::Auto => None,
            StreamQuality::Low => Some(240),
            StreamQuality::Medium => Some(480),
            StreamQuality::High => Some(720),
            StreamQuality::FullHd => Some(1080),
            StreamQuality::UltraHd => Some(2160),
        }
    }
}

/// Media processing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProcessingJob {
    /// Job ID
    pub id: Uuid,

    /// Source media file ID
    pub media_id: Uuid,

    /// Processing operation
    pub operation: ProcessingOperation,

    /// Job status
    pub status: JobStatus,

    /// Progress percentage (0-100)
    pub progress: f32,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,

    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,

    /// Output file ID (if applicable)
    pub output_id: Option<Uuid>,
}

impl MediaProcessingJob {
    /// Create a new processing job
    pub fn new(media_id: Uuid, operation: ProcessingOperation) -> Self {
        Self {
            id: Uuid::new_v4(),
            media_id,
            operation,
            status: JobStatus::Pending,
            progress: 0.0,
            error: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            output_id: None,
        }
    }
}

/// Processing operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessingOperation {
    /// Transcode to different format
    Transcode {
        target_format: String,
        target_codec: Option<String>,
        target_bitrate: Option<u32>,
    },
    /// Extract thumbnail
    GenerateThumbnail {
        timestamp_secs: f32,
        width: u32,
        height: u32,
    },
    /// Resize image/video
    Resize {
        width: u32,
        height: u32,
        maintain_aspect: bool,
    },
    /// Extract audio from video
    ExtractAudio { format: String },
    /// Compress file
    Compress {
        quality: u8, // 1-100
    },
    /// Generate streaming manifests
    PrepareStream { protocol: StreamProtocol },
    /// Extract metadata
    AnalyzeMetadata,
}

/// Job execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Media query/search parameters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaQuery {
    /// Filter by media type
    pub media_type: Option<MediaType>,

    /// Filter by tags
    pub tags: Vec<String>,

    /// Filter by source device
    pub device_id: Option<String>,

    /// Filter by date range
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,

    /// Search in filename/metadata
    pub search_term: Option<String>,

    /// Filter by status
    pub status: Option<MediaStatus>,

    /// Pagination
    pub limit: Option<usize>,
    pub offset: Option<usize>,

    /// Sorting
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
}

/// Fields available for sorting
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    UploadedAt,
    Filename,
    Size,
    Duration,
}

/// Sort order
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_file_creation() {
        let media = MediaFile::new("test.mp4".to_string(), MediaType::Video, "mp4".to_string());

        assert_eq!(media.filename, "test.mp4");
        assert_eq!(media.media_type, MediaType::Video);
        assert_eq!(media.mime_type, "video/mp4");
        assert_eq!(media.status, MediaStatus::Pending);
    }

    #[test]
    fn test_mime_type_detection() {
        let video = MediaFile::new("test.mp4".to_string(), MediaType::Video, "mp4".to_string());
        assert_eq!(video.mime_type, "video/mp4");

        let audio = MediaFile::new("test.mp3".to_string(), MediaType::Audio, "mp3".to_string());
        assert_eq!(audio.mime_type, "audio/mpeg");

        let image = MediaFile::new("test.png".to_string(), MediaType::Image, "png".to_string());
        assert_eq!(image.mime_type, "image/png");
    }

    #[test]
    fn test_media_dimensions() {
        let dims = MediaDimensions::new(1920, 1080);
        assert_eq!(dims.width, 1920);
        assert_eq!(dims.height, 1080);
        assert!(dims.is_full_hd());
        assert!(!dims.is_4k());
        assert!((dims.aspect_ratio() - 1.777).abs() < 0.01);
    }

    #[test]
    fn test_stream_quality_bitrate() {
        assert_eq!(StreamQuality::Low.bitrate_kbps(), 400);
        assert_eq!(StreamQuality::High.bitrate_kbps(), 2500);
        assert_eq!(StreamQuality::UltraHd.bitrate_kbps(), 15000);
    }

    #[test]
    fn test_stream_quality_height() {
        assert_eq!(StreamQuality::Low.height(), Some(240));
        assert_eq!(StreamQuality::High.height(), Some(720));
        assert_eq!(StreamQuality::FullHd.height(), Some(1080));
    }

    #[test]
    fn test_stream_config_creation() {
        let media_id = Uuid::new_v4();
        let config = StreamConfig::new(media_id, StreamProtocol::Hls);

        assert_eq!(config.media_id, media_id);
        assert_eq!(config.protocol, StreamProtocol::Hls);
        assert!(config.adaptive);
        assert!(!config.is_live);
    }

    #[test]
    fn test_live_stream_config() {
        let media_id = Uuid::new_v4();
        let config = StreamConfig::live(media_id);

        assert!(config.is_live);
        assert_eq!(config.protocol, StreamProtocol::WebRtc);
        assert_eq!(config.buffer_secs, 2.0);
    }

    #[test]
    fn test_processing_job_creation() {
        let media_id = Uuid::new_v4();
        let operation = ProcessingOperation::Transcode {
            target_format: "mp4".to_string(),
            target_codec: Some("H.264".to_string()),
            target_bitrate: Some(2500),
        };

        let job = MediaProcessingJob::new(media_id, operation);
        assert_eq!(job.media_id, media_id);
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.progress, 0.0);
    }
}
