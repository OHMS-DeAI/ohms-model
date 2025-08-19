use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap,
};
use std::cell::RefCell;
use crate::domain::*;
use candid::{encode_one, decode_one};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static MODEL_MANIFESTS: RefCell<StableBTreeMap<String, Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static MODEL_METADATA: RefCell<StableBTreeMap<String, Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    static CHUNK_STORAGE: RefCell<StableBTreeMap<String, Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static MODEL_STATS: RefCell<StableBTreeMap<String, Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );
}

fn chunk_key(model_id: &str, chunk_id: &str) -> String {
    format!("{}:{}", model_id, chunk_id)
}

const AUTH_UPLOADERS_KEY: &str = "__auth_uploaders";
const AUDIT_LOG_KEY: &str = "__audit_log";

// Model manifest storage
pub fn store_manifest(model_id: &str, manifest: &ModelManifest) -> ModelResult<()> {
    let manifest_data = encode_one(manifest).map_err(|_| ModelError::InvalidFormat)?;
    
    MODEL_MANIFESTS.with(|storage| {
        storage.borrow_mut().insert(model_id.to_string(), manifest_data);
    });
    
    Ok(())
}

pub fn get_manifest(model_id: &str) -> ModelResult<ModelManifest> {
    MODEL_MANIFESTS.with(|storage| {
        storage.borrow().get(&model_id.to_string())
            .ok_or(ModelError::NotFound)
            .and_then(|data| decode_one(&data).map_err(|_| ModelError::InvalidFormat))
    })
}

// Model metadata storage
pub fn store_model_meta(model_id: &str, meta: &ModelMeta) -> ModelResult<()> {
    let meta_data = encode_one(meta).map_err(|_| ModelError::InvalidFormat)?;
    
    MODEL_METADATA.with(|storage| {
        storage.borrow_mut().insert(model_id.to_string(), meta_data);
    });
    
    Ok(())
}

pub fn get_model_meta(model_id: &str) -> ModelResult<ModelMeta> {
    MODEL_METADATA.with(|storage| {
        storage.borrow().get(&model_id.to_string())
            .ok_or(ModelError::NotFound)
            .and_then(|data| decode_one(&data).map_err(|_| ModelError::InvalidFormat))
    })
}

// Chunk storage (namespaced by model)
pub fn store_chunk_for_model(model_id: &str, chunk_id: &str, chunk_data: Vec<u8>) -> ModelResult<()> {
    // Validate chunk size
    if chunk_data.len() > 2_097_152 { // 2 MiB limit
        return Err(ModelError::StorageFull);
    }
    
    CHUNK_STORAGE.with(|storage| {
        storage.borrow_mut().insert(chunk_key(model_id, chunk_id), chunk_data);
    });
    
    Ok(())
}

pub fn get_chunk_for_model(model_id: &str, chunk_id: &str) -> ModelResult<Vec<u8>> {
    CHUNK_STORAGE.with(|storage| {
        storage.borrow().get(&chunk_key(model_id, chunk_id))
            .ok_or(ModelError::NotFound)
    })
}

// Model listing and queries
pub fn list_models() -> Vec<String> {
    MODEL_MANIFESTS.with(|storage| {
        storage.borrow().iter().map(|(k, _)| k).collect()
    })
}

pub fn list_quantized_models() -> Vec<String> {
    let mut results = Vec::new();
    
    MODEL_MANIFESTS.with(|storage| {
        for (model_id, manifest_data) in storage.borrow().iter() {
            if let Ok(manifest) = decode_one::<ModelManifest>(&manifest_data) {
                if manifest.is_quantized() {
                    results.push(model_id);
                }
            }
        }
    });
    
    results
}

// Query by compression criteria
pub fn query_models_by_compression(min_ratio: f32) -> ModelResult<Vec<String>> {
    let mut results = Vec::new();
    
    MODEL_MANIFESTS.with(|storage| {
        for (model_id, manifest_data) in storage.borrow().iter() {
            if let Ok(manifest) = decode_one::<ModelManifest>(&manifest_data) {
                if let Some(ratio) = manifest.get_compression_ratio() {
                    if ratio >= min_ratio {
                        results.push(model_id);
                    }
                }
            }
        }
    });
    
    Ok(results)
}

pub fn query_models_by_size(max_size_mb: f32) -> ModelResult<Vec<String>> {
    let mut results = Vec::new();
    
    MODEL_MANIFESTS.with(|storage| {
        for (model_id, manifest_data) in storage.borrow().iter() {
            if let Ok(manifest) = decode_one::<ModelManifest>(&manifest_data) {
                if let Some(size_mb) = manifest.get_size_mb() {
                    if size_mb <= max_size_mb {
                        results.push(model_id);
                    }
                }
            }
        }
    });
    
    Ok(results)
}

// Global statistics
pub fn get_global_stats() -> ModelResult<ModelStats> {
    let mut total_models = 0u64;
    let mut quantized_models = 0u64;
    let mut total_compression_sum = 0.0f32;
    let mut total_capability_sum = 0.0f32;
    let mut total_size_saved = 0.0f32;
    
    MODEL_MANIFESTS.with(|storage| {
        total_models = storage.borrow().len() as u64;
        
        for (_, manifest_data) in storage.borrow().iter() {
            if let Ok(manifest) = decode_one::<ModelManifest>(&manifest_data) {
                if let Some(quantized_model) = &manifest.quantized_model {
                    quantized_models += 1;
                    total_compression_sum += quantized_model.compression_ratio;
                    total_capability_sum += quantized_model.bit_accuracy;
                    
                    // Calculate size saved (estimated)
                    let estimated_original_size_gb = 8.0; // 8GB typical for large models
                    let estimated_compressed_size_gb = estimated_original_size_gb / quantized_model.compression_ratio;
                    total_size_saved += estimated_original_size_gb - estimated_compressed_size_gb;
                }
            }
        }
    });
    
    let average_compression_ratio = if quantized_models > 0 {
        total_compression_sum / quantized_models as f32
    } else {
        0.0
    };
    
    let average_capability_retention = if quantized_models > 0 {
        total_capability_sum / quantized_models as f32
    } else {
        0.0
    };
    
    Ok(ModelStats {
        total_models,
        quantized_models,
        total_size_saved_gb: total_size_saved,
        total_energy_saved: total_size_saved * 71.0, // Estimated energy savings
        average_compression_ratio,
        average_capability_retention,
    })
}

// Cleanup deprecated models
pub fn cleanup_deprecated_models() -> ModelResult<u64> {
    let mut cleaned_count = 0u64;
    
    let deprecated_models: Vec<String> = MODEL_MANIFESTS.with(|storage| {
        let mut deprecated = Vec::new();
        for (model_id, manifest_data) in storage.borrow().iter() {
            if let Ok(manifest) = decode_one::<ModelManifest>(&manifest_data) {
                if matches!(manifest.state, ModelState::Deprecated) {
                    deprecated.push(model_id);
                }
            }
        }
        deprecated
    });
    
    // Remove chunks for deprecated models
    for model_id in deprecated_models {
        if let Ok(manifest) = get_manifest(&model_id) {
            for chunk in &manifest.chunks {
                CHUNK_STORAGE.with(|storage| {
                    storage.borrow_mut().remove(&chunk_key(&model_id, &chunk.id));
                });
                cleaned_count += 1;
            }
        }
    }
    
    Ok(cleaned_count)
}

// Authorized uploaders persistence
pub fn get_authorized_uploaders() -> Vec<String> {
    MODEL_STATS.with(|storage| {
        storage
            .borrow()
            .get(&AUTH_UPLOADERS_KEY.to_string())
            .and_then(|data| decode_one::<Vec<String>>(&data).ok())
            .unwrap_or_default()
    })
}

pub fn set_authorized_uploaders(uploaders: &Vec<String>) -> ModelResult<()> {
    let data = encode_one(uploaders).map_err(|_| ModelError::InvalidFormat)?;
    MODEL_STATS.with(|storage| {
        storage.borrow_mut().insert(AUTH_UPLOADERS_KEY.to_string(), data);
    });
    Ok(())
}

// Audit log persistence (simple append whole vector)
pub fn append_audit_event(event: &AuditEvent) -> ModelResult<()> {
    let mut log = get_audit_log();
    log.push(event.clone());
    let data = encode_one(&log).map_err(|_| ModelError::InvalidFormat)?;
    MODEL_STATS.with(|storage| {
        storage.borrow_mut().insert(AUDIT_LOG_KEY.to_string(), data);
    });
    Ok(())
}

pub fn get_audit_log() -> Vec<AuditEvent> {
    MODEL_STATS.with(|storage| {
        storage
            .borrow()
            .get(&AUDIT_LOG_KEY.to_string())
            .and_then(|data| decode_one::<Vec<AuditEvent>>(&data).ok())
            .unwrap_or_default()
    })
}