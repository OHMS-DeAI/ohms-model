use crate::{domain::*, services::*};
use candid::{candid_method, CandidType, Deserialize};
use ic_cdk::{api::caller, query, update};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade};
use serde::Serialize;
use std::cell::RefCell;

thread_local! {
    static REPOSITORY: RefCell<ModelRepository> = RefCell::new(ModelRepository::new());
}

#[init]
fn init() {
    let admin = caller().to_text();
    REPOSITORY.with(|repo| {
        repo.borrow_mut().add_authorized_uploader(admin);
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    // Persist authorized uploaders list before upgrade
    REPOSITORY.with(|repo| {
        let repo_ref = repo.borrow();
        let _ = crate::services::storage::set_authorized_uploaders(&repo_ref.authorized_uploaders);
    });
}

#[post_upgrade]
fn post_upgrade() {
    // Restore authorized uploaders list from stable memory
    let uploaders = crate::services::storage::get_authorized_uploaders();
    REPOSITORY.with(|repo| {
        let mut r = repo.borrow_mut();
        for u in uploaders {
            r.add_authorized_uploader(u);
        }
    });
}

// Core model operations
#[update]
#[candid_method(update)]
fn submit_model(upload: ModelUpload) -> Result<String, String> {
    let actor = caller().to_text();
    
    REPOSITORY.with(|repo| {
        repo.borrow_mut().submit_model(upload, actor)
    })?;
    
    Ok("Model submitted successfully".to_string())
}

#[update]
#[candid_method(update)]
fn submit_quantized_model(
    model_id: String,
    source_model: String,
    quantized_model: NOVAQModel,
    verification: NOVAQVerificationReport,
) -> Result<String, String> {
    let actor = caller().to_text();
    
    // Create upload from quantized model
    let upload = ModelUpload::from_quantized_model(
        model_id,
        source_model,
        quantized_model,
        verification,
    );
    
    REPOSITORY.with(|repo| {
        repo.borrow_mut().submit_model(upload, actor)
    })?;
    
    Ok("Quantized model submitted successfully".to_string())
}

#[update]  
#[candid_method(update)]
fn activate_model(model_id: ModelId) -> Result<String, String> {
    let actor = caller().to_text();
    
    REPOSITORY.with(|repo| {
        repo.borrow_mut().activate_model(&model_id, actor)
    })?;
    
    Ok("Model activated successfully".to_string())
}

#[update]
#[candid_method(update)]
fn deprecate_model(model_id: ModelId) -> Result<String, String> {
    let actor = caller().to_text();
    
    REPOSITORY.with(|repo| {
        repo.borrow_mut().deprecate_model(&model_id, actor)
    })?;
    
    Ok("Model deprecated successfully".to_string())
}

// Query operations
#[query]
#[candid_method(query)]
fn get_manifest(model_id: ModelId) -> Option<ModelManifest> {
    // Prefer stable storage read for source of truth
    crate::services::storage::get_manifest(&model_id.0).ok()
}

#[query]
#[candid_method(query)]
fn get_model_meta(model_id: ModelId) -> Option<ModelMeta> {
    crate::services::storage::get_model_meta(&model_id.0).ok()
}

#[query]
#[candid_method(query)]
fn get_chunk(model_id: ModelId, chunk_id: String) -> Option<Vec<u8>> {
    let actor = caller().to_text();
    REPOSITORY.with(|repo| repo.borrow_mut().get_chunk(&model_id, &chunk_id, actor))
}

#[query]
#[candid_method(query)]  
fn list_models(state_filter: Option<ModelState>) -> Vec<ModelManifest> {
    // Read all manifests from stable and filter in-memory for state
    let ids = crate::services::storage::list_models();
    let mut out = Vec::new();
    for id in ids {
        if let Ok(m) = crate::services::storage::get_manifest(&id) {
            if let Some(filter) = &state_filter {
                if std::mem::discriminant(&m.state) != std::mem::discriminant(filter) {
                    continue;
                }
            }
            out.push(m);
        }
    }
    out
}

#[query]
#[candid_method(query)]
fn list_quantized_models() -> Vec<ModelManifest> {
    let ids = crate::services::storage::list_quantized_models();
    ids.into_iter()
        .filter_map(|id| crate::services::storage::get_manifest(&id).ok())
        .collect()
}

// Enhanced queries for quantized models
#[query]
#[candid_method(query)]
fn query_models_by_compression(min_ratio: f32) -> Vec<String> {
    storage::query_models_by_compression(min_ratio).unwrap_or_default()
}

#[query]
#[candid_method(query)]
fn query_models_by_size(max_size_mb: f32) -> Vec<String> {
    storage::query_models_by_size(max_size_mb).unwrap_or_default()
}

#[query]
#[candid_method(query)]
fn get_global_stats() -> ModelStats {
    storage::get_global_stats().unwrap_or(ModelStats {
        total_models: 0,
        quantized_models: 0,
        total_size_saved_gb: 0.0,
        total_energy_saved: 0.0,
        average_compression_ratio: 0.0,
        average_capability_retention: 0.0,
    })
}

// Audit operations
#[query]
#[candid_method(query)]
fn get_audit_log() -> Vec<AuditEvent> {
    REPOSITORY.with(|repo| {
        repo.borrow().get_audit_log().to_vec()
    })
}

// Admin operations
#[update]
#[candid_method(update)]
fn add_authorized_uploader(uploader: String) -> Result<String, String> {
    let actor = caller().to_text();
    
    REPOSITORY.with(|repo| {
        let repo_ref = repo.borrow();
        if !repo_ref.authorized_uploaders.contains(&actor) {
            return Err("Not authorized to add uploaders".to_string());
        }
        Ok(())
    })?;
    
    REPOSITORY.with(|repo| {
        repo.borrow_mut().add_authorized_uploader(uploader);
    });
    
    Ok("Authorized uploader added".to_string())
}

#[update]
#[candid_method(update)]
fn cleanup_deprecated_models() -> Result<String, String> {
    let actor = caller().to_text();
    
    // Check authorization
    REPOSITORY.with(|repo| {
        let repo_ref = repo.borrow();
        if !repo_ref.authorized_uploaders.contains(&actor) {
            return Err("Not authorized to cleanup models".to_string());
        }
        Ok(())
    })?;
    
    let cleaned_count = storage::cleanup_deprecated_models()
        .map_err(|e| format!("Cleanup failed: {:?}", e))?;
    
    Ok(format!("Cleaned up {} chunks from deprecated models", cleaned_count))
}

// Health and utility
#[query]
#[candid_method(query)]
fn health() -> String {
    "OK".to_string()
}

#[query]
#[candid_method(query)]
fn get_compression_stats() -> String {
    let stats = get_global_stats();
    format!(
        "Models: {} total, {} quantized | Avg compression: {:.1}x | Avg capability: {:.1}% | Size saved: {:.1} GB",
        stats.total_models,
        stats.quantized_models,
        stats.average_compression_ratio,
        stats.average_capability_retention,
        stats.total_size_saved_gb
    )
}

// Generate Candid interface
candid::export_service!();

#[query]
#[candid_method(query)]
fn __get_candid_interface_tmp_hack() -> String {
    __export_service()
}