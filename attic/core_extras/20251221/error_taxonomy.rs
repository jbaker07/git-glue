/// Production error taxonomy for structured logging and metrics

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    SegmentMissing,
    SegmentHashMismatch,
    SegmentJsonCorrupt,
    DbError,
    PlaybookInvalid,
    EvidenceMissing,
    Unknown,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::SegmentMissing => "segment_missing",
            ErrorCode::SegmentHashMismatch => "segment_hash_mismatch",
            ErrorCode::SegmentJsonCorrupt => "segment_json_corrupt",
            ErrorCode::DbError => "db_error",
            ErrorCode::PlaybookInvalid => "playbook_invalid",
            ErrorCode::EvidenceMissing => "evidence_missing",
            ErrorCode::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub code: ErrorCode,
    pub message: String,
    pub ptr: Option<(String, u32)>,
}

impl ErrorReport {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        ErrorReport {
            code,
            message: message.into(),
            ptr: None,
        }
    }

    pub fn with_ptr(code: ErrorCode, message: impl Into<String>, ptr: (String, u32)) -> Self {
        ErrorReport {
            code,
            message: message.into(),
            ptr: Some(ptr),
        }
    }
}
