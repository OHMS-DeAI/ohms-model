use candid::{CandidType, Deserialize};
use serde::Serialize;
use sha2::Digest;

// Minimal inline definitions to avoid heavy engine dependency in the canister.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SuperQuantizedModel {
    pub architecture: QuantArch,
    pub compressed_model: CompressedModel,
    pub verification: Verification,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct QuantArch {
    pub family: String,
    pub layers: u32,
    pub hidden_size: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CompressedModel {
    pub data: Vec<u8>,
    pub metadata: CompressedMeta,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CompressedMeta {
    pub compression_ratio: f32,
    pub original_size: u64,
    pub compressed_size: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Verification {
    pub bit_accuracy: f32,
}

pub type APQVerificationReport = Verification;

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

// Enhanced model manifest
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ModelManifest {
    pub model_id: ModelId,
    pub version: String,
    pub chunks: Vec<ChunkInfo>,
    pub digest: String,
    pub state: ModelState,
    pub uploaded_at: u64,
    pub activated_at: Option<u64>,
    // Quantization info
    pub compression_type: CompressionType,
    pub quantized_model: Option<SuperQuantizedModel>, // Direct use of ohms-adaptq type
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum CompressionType {
    LegacyAPQ,
    SuperAPQ,
    Uncompressed,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ModelMeta {
    pub family: String,
    pub arch: String,
    pub tokenizer_id: String,
    pub vocab_size: u32,
    pub ctx_window: u32,
    pub license: String,
    pub quantization_info: QuantizationInfo,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct QuantizationInfo {
    pub method: String,
    pub quantizer_version: String,
    pub quantization_date: u64,
    pub source_model: String,
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
    pub verification_report: Option<APQVerificationReport>, // Use ohms-adaptq type
}

// Enhanced badge system
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Badge {
    pub badge_type: BadgeType,
    pub granted_at: u64,
    pub granted_by: String,
    pub metadata: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum BadgeType {
    VerifiedQuant,
    Reproducible,
    GovernanceApproved,
    CommunityTested,
    // Advanced quantization badges
    HighCompression,
    ZeroCost,
    EnergyEfficient,
    UniversalCompatible,
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
    Quantization,
    Verification,
}

// Query types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ModelQuery {
    pub compression_type: Option<CompressionType>,
    pub min_compression_ratio: Option<f32>,
    pub min_capability_retention: Option<f32>,
    pub max_size_mb: Option<f32>,
    pub architecture: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ModelStats {
    pub total_models: u64,
    pub quantized_models: u64,
    pub total_size_saved_gb: f32,
    pub total_energy_saved: f32,
    pub average_compression_ratio: f32,
    pub average_capability_retention: f32,
}

// Error types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ModelError {
    NotFound,
    InvalidState,
    CompressionFailed,
    VerificationFailed,
    StorageFull,
    UnauthorizedAccess,
    InvalidFormat,
}

// Result type
pub type ModelResult<T> = Result<T, ModelError>;

// Helper methods
impl ModelManifest {
    /// Check if model is quantized
    pub fn is_quantized(&self) -> bool {
        matches!(self.compression_type, CompressionType::SuperAPQ | CompressionType::LegacyAPQ)
    }
    
    /// Get compression ratio if available
    pub fn get_compression_ratio(&self) -> Option<f32> {
        self.quantized_model.as_ref()
            .map(|model| model.compressed_model.metadata.compression_ratio)
    }
    
    /// Get compressed size in MB
    pub fn get_size_mb(&self) -> Option<f32> {
        self.quantized_model.as_ref()
            .map(|model| model.compressed_model.metadata.compressed_size as f32 / 1_000_000.0)
    }
}

impl ModelUpload {
    /// Create upload from quantized model
    pub fn from_quantized_model(
        model_id: String,
        source_model: String,
        quantized_model: SuperQuantizedModel,
        verification: APQVerificationReport,
    ) -> Self {
        let model_id = ModelId(model_id);
        let timestamp = ic_cdk::api::time();

        // Split compressed bytes into 2 MiB shards
        let bytes = quantized_model.compressed_model.data.clone();
        let max_chunk: usize = 2 * 1024 * 1024;
        let mut chunks: Vec<ChunkData> = Vec::new();
        let mut infos: Vec<ChunkInfo> = Vec::new();
        let mut offset: u64 = 0;
        let mut hasher = sha2::Sha256::new();
        for (idx, part) in bytes.chunks(max_chunk).enumerate() {
            let chunk_id = format!("apq-{:06}", idx);
            let sha = sha2::Sha256::digest(part);
            hasher.update(sha);
            chunks.push(ChunkData { chunk_id: chunk_id.clone(), data: part.to_vec() });
            infos.push(ChunkInfo {
                id: chunk_id,
                offset,
                size: part.len() as u64,
                sha256: hex::encode(sha),
            });
            offset += part.len() as u64;
        }
        let digest = hex::encode(hasher.finalize());

        let manifest = ModelManifest {
            model_id: model_id.clone(),
            version: "2.0.0".to_string(),
            chunks: infos,
            digest,
            state: ModelState::Pending,
            uploaded_at: timestamp,
            activated_at: None,
            compression_type: CompressionType::SuperAPQ,
            // Keep metadata but do not rely on embedded bytes for serving
            quantized_model: Some(SuperQuantizedModel {
                architecture: quantized_model.architecture.clone(),
                compressed_model: CompressedModel {
                    // Avoid duplicating large data in manifest; store empty to reduce RAM footprint
                    data: Vec::new(),
                    metadata: quantized_model.compressed_model.metadata.clone(),
                },
                verification: quantized_model.verification.clone(),
            }),
        };

        let meta = ModelMeta {
            family: manifest
                .quantized_model
                .as_ref()
                .map(|q| q.architecture.family.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            arch: format!("{}-layers", quantized_model.architecture.layers),
            tokenizer_id: source_model.clone(),
            vocab_size: 32000, // Default
            ctx_window: quantized_model.architecture.hidden_size as u32,
            license: "MIT".to_string(),
            quantization_info: QuantizationInfo {
                method: "ohms-adaptq-v2".to_string(),
                quantizer_version: "2.0.0".to_string(),
                quantization_date: timestamp,
                source_model,
            },
        };

        Self {
            model_id,
            manifest,
            meta,
            chunks,
            signature: None,
            verification_report: Some(verification),
        }
    }
}