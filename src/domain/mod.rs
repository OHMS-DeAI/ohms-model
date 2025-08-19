use candid::{CandidType, Deserialize};
use serde::Serialize;
use sha2::Digest;

// NOVAQ types defined locally for WASM compatibility
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct NOVAQConfig {
    pub target_bits: f32,
    pub num_subspaces: usize,
    pub codebook_size_l1: usize,
    pub codebook_size_l2: usize,
    pub outlier_threshold: f32,
    pub teacher_model_path: Option<String>,
    pub refinement_iterations: usize,
    pub kl_weight: f32,
    pub cosine_weight: f32,
    pub learning_rate: f32,
    pub seed: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct NOVAQModel {
    pub config: NOVAQConfig,
    pub compression_ratio: f32,
    pub bit_accuracy: f32,
    pub vector_codebooks: Vec<Vec<Vec<f32>>>, // Simplified for WASM
    pub quantization_indices: Vec<Vec<u8>>,
    pub weight_shapes: Vec<(String, Vec<usize>)>,
    pub normalization_metadata: Vec<f32>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Verification {
    pub bit_accuracy: f32,
}

pub type NOVAQVerificationReport = Verification;

// Candid-compatible NOVAQ model wrapper
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct NOVAQModelCandid {
    pub config: NOVAQConfigCandid,
    pub compression_ratio: f32,
    pub bit_accuracy: f32,
    pub vector_codebooks: Vec<Vec<Vec<f32>>>, // Vec<Vec<CodebookEntry.centroid>>
    pub quantization_indices: Vec<Vec<u8>>,   // Vec<Vec<u8>>
    pub weight_shapes: Vec<(String, Vec<u32>)>, // Vec<(name, shape)>
    pub normalization_metadata: Vec<f32>,     // Flattened NormalizationMetadata
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct NOVAQConfigCandid {
    pub target_bits: f32,
    pub num_subspaces: u32,
    pub codebook_size_l1: u32,
    pub codebook_size_l2: u32,
    pub outlier_threshold: f32,
    pub teacher_model_path: Option<String>,
    pub refinement_iterations: u32,
    pub kl_weight: f32,
    pub cosine_weight: f32,
    pub learning_rate: f32,
    pub seed: u64,
}

impl From<NOVAQModel> for NOVAQModelCandid {
    fn from(model: NOVAQModel) -> Self {
        // Convert weight shapes to Candid format
        let weight_shapes: Vec<(String, Vec<u32>)> = model.weight_shapes
            .iter()
            .map(|(name, shape)| (name.clone(), shape.iter().map(|&s| s as u32).collect()))
            .collect();

        Self {
            config: NOVAQConfigCandid {
                target_bits: model.config.target_bits,
                num_subspaces: model.config.num_subspaces as u32,
                codebook_size_l1: model.config.codebook_size_l1 as u32,
                codebook_size_l2: model.config.codebook_size_l2 as u32,
                outlier_threshold: model.config.outlier_threshold,
                teacher_model_path: model.config.teacher_model_path.clone(),
                refinement_iterations: model.config.refinement_iterations as u32,
                kl_weight: model.config.kl_weight,
                cosine_weight: model.config.cosine_weight,
                learning_rate: model.config.learning_rate,
                seed: model.config.seed,
            },
            compression_ratio: model.compression_ratio,
            bit_accuracy: model.bit_accuracy,
            vector_codebooks: model.vector_codebooks,
            quantization_indices: model.quantization_indices,
            weight_shapes,
            normalization_metadata: model.normalization_metadata,
        }
    }
}

impl From<NOVAQModelCandid> for NOVAQModel {
    fn from(candid_model: NOVAQModelCandid) -> Self {
        // Convert weight shapes back to Vec<(String, Vec<usize>)>
        let weight_shapes: Vec<(String, Vec<usize>)> = candid_model.weight_shapes
            .iter()
            .map(|(name, shape)| (name.clone(), shape.iter().map(|&s| s as usize).collect()))
            .collect();
        
        Self {
            config: NOVAQConfig {
                target_bits: candid_model.config.target_bits,
                num_subspaces: candid_model.config.num_subspaces as usize,
                codebook_size_l1: candid_model.config.codebook_size_l1 as usize,
                codebook_size_l2: candid_model.config.codebook_size_l2 as usize,
                outlier_threshold: candid_model.config.outlier_threshold,
                teacher_model_path: candid_model.config.teacher_model_path.clone(),
                refinement_iterations: candid_model.config.refinement_iterations as usize,
                kl_weight: candid_model.config.kl_weight,
                cosine_weight: candid_model.config.cosine_weight,
                learning_rate: candid_model.config.learning_rate,
                seed: candid_model.config.seed,
            },
            compression_ratio: candid_model.compression_ratio,
            bit_accuracy: candid_model.bit_accuracy,
            vector_codebooks: candid_model.vector_codebooks,
            quantization_indices: candid_model.quantization_indices,
            weight_shapes,
            normalization_metadata: candid_model.normalization_metadata,
        }
    }
}

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
    pub quantized_model: Option<NOVAQModelCandid>, // Candid-compatible wrapper
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

        // Create compressed model data from NOVAQ model
        let candid_model = NOVAQModelCandid::from(quantized_model.clone());
        let bytes = bincode::serialize(&candid_model).unwrap_or_default();
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
            quantized_model: Some(NOVAQModelCandid::from(quantized_model.clone())),
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