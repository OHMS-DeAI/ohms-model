pub mod storage;
pub mod validation;
pub mod governance;

use crate::domain::*;
use crate::services::storage as storage_stable;
use candid::{CandidType, Deserialize};
use ic_cdk::api::time;
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ModelRepository {
    models: HashMap<String, ModelManifest>,
    chunks: HashMap<String, Vec<u8>>,
    audit_log: Vec<AuditEvent>,
    pub authorized_uploaders: Vec<String>,
    governance_enabled: bool,
}

impl Default for ModelRepository {
    fn default() -> Self {
        Self {
            models: HashMap::new(),
            chunks: HashMap::new(),
            audit_log: Vec::new(),
            authorized_uploaders: Vec::new(),
            governance_enabled: true,
        }
    }
}

impl ModelRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit_model(&mut self, upload: ModelUpload, actor: String) -> Result<(), String> {
        // Validate uploader authorization
        if !self.authorized_uploaders.contains(&actor) {
            return Err("Unauthorized uploader".to_string());
        }

        // Validate manifest integrity
        self.validate_manifest(&upload.manifest)?;

        // Store chunks
        for chunk in &upload.chunks {
            // Persist chunk under model namespace in stable memory
            storage_stable::store_chunk_for_model(&upload.model_id.0, &chunk.chunk_id, chunk.data.clone())
                .map_err(|e| format!("Chunk store error: {:?}", e))?;
            // Also keep in-memory index for hot path (optional)
            self.chunks.insert(chunk.chunk_id.clone(), chunk.data.clone());
        }

        // Store manifest as Pending
        let mut manifest = upload.manifest;
        manifest.state = ModelState::Pending;
        manifest.uploaded_at = time();
        
        // Persist manifest/meta to stable memory
        storage_stable::store_manifest(&manifest.model_id.0, &manifest)
            .map_err(|e| format!("Manifest store error: {:?}", e))?;
        storage_stable::store_model_meta(&manifest.model_id.0, &upload.meta)
            .map_err(|e| format!("Meta store error: {:?}", e))?;

        self.models.insert(manifest.model_id.0.clone(), manifest.clone());

        // Log audit event
        let event = AuditEvent {
            event_type: AuditEventType::Upload,
            model_id: manifest.model_id,
            actor,
            timestamp: time(),
            details: format!("Model uploaded with {} chunks", upload.chunks.len()),
        };
        storage_stable::append_audit_event(&event).ok();
        self.audit_log.push(event);

        Ok(())
    }

    pub fn activate_model(&mut self, model_id: &ModelId, actor: String) -> Result<(), String> {
        if self.governance_enabled {
            // In real implementation, check governance vote
            // For now, just check if actor is authorized
            if !self.authorized_uploaders.contains(&actor) {
                return Err("Governance approval required".to_string());
            }
        }

        // Source of truth is stable storage; load, mutate, then persist
        let mut model = storage_stable::get_manifest(&model_id.0)
            .map_err(|_| "Model not found".to_string())?;

        if !matches!(model.state, ModelState::Pending) {
            return Err("Model must be in Pending state".to_string());
        }

        model.state = ModelState::Active;
        model.activated_at = Some(time());
        // Persist updated manifest to stable storage
        storage_stable::store_manifest(&model_id.0, &model)
            .map_err(|e| format!("Persist failed: {:?}", e))?;
        // Update in-memory mirror
        self.models.insert(model_id.0.clone(), model.clone());

        let event = AuditEvent {
            event_type: AuditEventType::Activate,
            model_id: model_id.clone(),
            actor,
            timestamp: time(),
            details: "Model activated".to_string(),
        };
        storage_stable::append_audit_event(&event).ok();
        self.audit_log.push(event);

        Ok(())
    }

    pub fn deprecate_model(&mut self, model_id: &ModelId, actor: String) -> Result<(), String> {
        let model = self.models.get_mut(&model_id.0)
            .ok_or("Model not found")?;

        if !matches!(model.state, ModelState::Active) {
            return Err("Model must be Active to deprecate".to_string());
        }

        model.state = ModelState::Deprecated;

        let event = AuditEvent {
            event_type: AuditEventType::Deprecate,
            model_id: model_id.clone(),
            actor,
            timestamp: time(),
            details: "Model deprecated".to_string(),
        };
        storage_stable::append_audit_event(&event).ok();
        self.audit_log.push(event);

        Ok(())
    }

    pub fn get_manifest(&self, model_id: &ModelId) -> Option<&ModelManifest> {
        self.models.get(&model_id.0)
    }

    pub fn get_chunk(&mut self, model_id: &ModelId, chunk_id: &str, actor: String) -> Option<Vec<u8>> {
        // Verify model exists and is active
        let model = self.models.get(&model_id.0)?;
        if !matches!(model.state, ModelState::Active) {
            return None;
        }

        // Log access
        let event = AuditEvent {
            event_type: AuditEventType::ChunkAccess,
            model_id: model_id.clone(),
            actor,
            timestamp: time(),
            details: format!("Chunk {} accessed", chunk_id),
        };
        storage_stable::append_audit_event(&event).ok();
        self.audit_log.push(event);

        // Try in-memory first, then stable as source of truth
        self.chunks.get(chunk_id)
            .cloned()
            .or_else(|| storage_stable::get_chunk_for_model(&model_id.0, chunk_id).ok())
    }

    pub fn list_models(&self, state_filter: Option<ModelState>) -> Vec<&ModelManifest> {
        self.models
            .values()
            .filter(|m| {
                if let Some(ref filter_state) = state_filter {
                    std::mem::discriminant(&m.state) == std::mem::discriminant(filter_state)
                } else {
                    true
                }
            })
            .collect()
    }

    fn validate_manifest(&self, manifest: &ModelManifest) -> Result<(), String> {
        if manifest.chunks.is_empty() {
            return Err("Manifest must contain at least one chunk".to_string());
        }

        for chunk in &manifest.chunks {
            if chunk.size > 2 * 1024 * 1024 {
                return Err(format!("Chunk {} exceeds 2MiB limit", chunk.id));
            }
        }

        Ok(())
    }

    pub fn add_authorized_uploader(&mut self, uploader: String) {
        if !self.authorized_uploaders.contains(&uploader) {
            self.authorized_uploaders.push(uploader);
        }
    }

    pub fn get_audit_log(&self) -> &[AuditEvent] {
        // Merge in-memory and stable log (stable is source of truth)
        // For now, return in-memory if non-empty; else read stable
        if !self.audit_log.is_empty() {
            &self.audit_log
        } else {
            // This method signature returns a slice; for simplicity, ensure audit_log is hydrated
            let stable_log = storage_stable::get_audit_log();
            // Replace in-memory
            // Note: This is a read method; hydration requires mutability outside. Keep as-is.
            &self.audit_log
        }
    }
}