use candid::{CandidType, Deserialize};
use serde::Serialize;
use sha2::Digest;

// NOVAQ types imported from ohms-adaptq
use ohms_adaptq::{NOVAQModel, NOVAQConfig, WeightMatrix};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Verification {
    pub bit_accuracy: f32,
}

pub type NOVAQVerificationReport = Verification;

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
    pub quantized_model: Option<NOVAQModel>, // Direct use of ohms-adaptq type
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum CompressionType {
    NOVAQ,
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
    pub verification_report: Option<NOVAQVerificationReport>, // Use ohms-adaptq type
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
        matches!(self.compression_type, CompressionType::NOVAQ)
    }
    
    /// Get compression ratio if available
    pub fn get_compression_ratio(&self) -> Option<f32> {
        self.quantized_model.as_ref()
            .map(|model| model.compression_ratio)
    }
    
    /// Get compressed size in MB (estimated from compression ratio)
    pub fn get_size_mb(&self) -> Option<f32> {
        // Estimate size based on compression ratio and typical model sizes
        self.quantized_model.as_ref()
            .map(|model| {
                let estimated_original_size = 8000.0; // 8GB typical for large models
                estimated_original_size * (1.0 - model.compression_ratio / 100.0)
            })
    }
}

impl ModelUpload {
    /// Create upload from quantized model
    pub fn from_quantized_model(
        model_id: String,
        source_model: String,
        quantized_model: NOVAQModel,
        verification: NOVAQVerificationReport,
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
            let chunk_id = format!("novaq-{:06}", idx);
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
            compression_type: CompressionType::NOVAQ,
            // Keep metadata but do not rely on embedded bytes for serving
            quantized_model: Some(NOVAQModel {
                config: quantized_model.config.clone(),
                compression_ratio: quantized_model.compression_ratio,
                bit_accuracy: quantized_model.bit_accuracy,
                codebooks: quantized_model.codebooks.clone(),
                outliers: quantized_model.outliers.clone(),
            }),
        };

        let meta = ModelMeta {
            family: "novaq".to_string(),
            arch: format!("novaq-{}", quantized_model.config.num_subspaces),
            tokenizer_id: source_model.clone(),
            vocab_size: 32000, // Default
            ctx_window: 4096, // Default context window for NOVAQ models
            license: "MIT".to_string(),
            quantization_info: QuantizationInfo {
                method: "novaq-v2".to_string(),
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