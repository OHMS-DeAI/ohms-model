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
    // Initialize with admin as authorized uploader
    let admin = caller().to_text();
    REPOSITORY.with(|repo| {
        repo.borrow_mut().add_authorized_uploader(admin);
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    // In production, serialize state to stable memory
}

#[post_upgrade]
fn post_upgrade() {
    // In production, deserialize state from stable memory
}

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

#[query]
#[candid_method(query)]
fn get_manifest(model_id: ModelId) -> Option<ModelManifest> {
    REPOSITORY.with(|repo| {
        repo.borrow().get_manifest(&model_id).cloned()
    })
}

#[query]
#[candid_method(query)]
fn get_chunk(model_id: ModelId, chunk_id: String) -> Option<Vec<u8>> {
    let actor = caller().to_text();
    
    REPOSITORY.with(|repo| {
        repo.borrow_mut().get_chunk(&model_id, &chunk_id, actor)
    })
}

#[query]
#[candid_method(query)]  
fn list_models(state_filter: Option<ModelState>) -> Vec<ModelManifest> {
    REPOSITORY.with(|repo| {
        repo.borrow().list_models(state_filter).into_iter().cloned().collect()
    })
}

#[query]
#[candid_method(query)]
fn get_audit_log() -> Vec<AuditEvent> {
    REPOSITORY.with(|repo| {
        repo.borrow().get_audit_log().to_vec()
    })
}

#[update]
#[candid_method(update)]
fn add_authorized_uploader(uploader: String) -> Result<String, String> {
    let actor = caller().to_text();
    
    // Only existing authorized uploaders can add new ones
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

// Health check endpoint
#[query]
#[candid_method(query)]
fn health() -> String {
    "OK".to_string()
}

// Generate Candid interface
candid::export_service!();

#[query]
#[candid_method(query)]
fn __get_candid_interface_tmp_hack() -> String {
    __export_service()
}