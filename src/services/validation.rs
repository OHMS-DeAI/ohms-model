use crate::domain::*;
use sha2::{Digest, Sha256};

pub fn validate_chunk_integrity(chunk: &ChunkData) -> Result<(), String> {
    if chunk.data.len() > 2 * 1024 * 1024 {
        return Err("Chunk exceeds 2MiB size limit".to_string());
    }

    if chunk.data.is_empty() {
        return Err("Chunk cannot be empty".to_string());
    }

    Ok(())
}

pub fn validate_manifest_hashes(manifest: &ModelManifest, chunks: &[ChunkData]) -> Result<(), String> {
    if manifest.chunks.len() != chunks.len() {
        return Err("Chunk count mismatch between manifest and data".to_string());
    }

    for (manifest_chunk, actual_chunk) in manifest.chunks.iter().zip(chunks.iter()) {
        if manifest_chunk.id != actual_chunk.chunk_id {
            return Err(format!("Chunk ID mismatch: {} != {}", manifest_chunk.id, actual_chunk.chunk_id));
        }

        if manifest_chunk.size != actual_chunk.data.len() as u64 {
            return Err(format!("Chunk size mismatch for {}: {} != {}", 
                manifest_chunk.id, manifest_chunk.size, actual_chunk.data.len()));
        }

        // Verify SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(&actual_chunk.data);
        let calculated_hash = hex::encode(hasher.finalize());

        if manifest_chunk.sha256 != calculated_hash {
            return Err(format!("Hash mismatch for chunk {}: {} != {}", 
                manifest_chunk.id, manifest_chunk.sha256, calculated_hash));
        }
    }

    Ok(())
}

pub fn calculate_manifest_digest(manifest: &ModelManifest) -> String {
    let mut hasher = Sha256::new();
    
    // Hash all chunk information
    for chunk in &manifest.chunks {
        hasher.update(chunk.id.as_bytes());
        hasher.update(&chunk.offset.to_le_bytes());
        hasher.update(&chunk.size.to_le_bytes());
        hasher.update(chunk.sha256.as_bytes());
    }
    
    hex::encode(hasher.finalize())
}

pub fn validate_model_meta(meta: &ModelMeta) -> Result<(), String> {
    if meta.family.is_empty() {
        return Err("Model family cannot be empty".to_string());
    }

    if meta.arch.is_empty() {
        return Err("Model architecture cannot be empty".to_string());
    }

    if meta.vocab_size == 0 {
        return Err("Vocabulary size must be greater than 0".to_string());
    }

    if meta.ctx_window == 0 {
        return Err("Context window must be greater than 0".to_string());
    }

    Ok(())
}