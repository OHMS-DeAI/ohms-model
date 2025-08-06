use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ModelId(pub String);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ModelState {
    Pending,
    Active,
    Deprecated,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ChunkInfo {
    pub id: String,
    pub offset: u64,
    pub size: u64,
    pub sha256: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ModelManifest {
    pub model_id: ModelId,
    pub version: String,
    pub chunks: Vec<ChunkInfo>,
    pub digest: String,
    pub state: ModelState,
    pub uploaded_at: u64,
    pub activated_at: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ModelMeta {
    pub family: String,
    pub arch: String,
    pub tokenizer_id: String,
    pub vocab_size: u32,
    pub ctx_window: u32,
    pub license: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ChunkData {
    pub chunk_id: String,
    pub data: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ModelUpload {
    pub model_id: ModelId,
    pub manifest: ModelManifest,
    pub meta: ModelMeta,
    pub chunks: Vec<ChunkData>,
    pub signature: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Badge {
    pub badge_type: BadgeType,
    pub granted_at: u64,
    pub granted_by: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum BadgeType {
    VerifiedQuant,
    Reproducible,
    GovernanceApproved,
    CommunityTested,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub model_id: ModelId,
    pub actor: String,
    pub timestamp: u64,
    pub details: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum AuditEventType {
    Upload,
    Activate,
    Deprecate,
    ChunkAccess,
    BadgeGrant,
}