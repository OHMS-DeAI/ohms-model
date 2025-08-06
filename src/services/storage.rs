use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap,
};
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static MODEL_STORAGE: RefCell<StableBTreeMap<String, Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static CHUNK_STORAGE: RefCell<StableBTreeMap<String, Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );
}

pub fn store_manifest(model_id: &str, manifest_data: Vec<u8>) {
    MODEL_STORAGE.with(|storage| {
        storage.borrow_mut().insert(model_id.to_string(), manifest_data);
    });
}

pub fn get_manifest(model_id: &str) -> Option<Vec<u8>> {
    MODEL_STORAGE.with(|storage| {
        storage.borrow().get(&model_id.to_string())
    })
}

pub fn store_chunk(chunk_id: &str, chunk_data: Vec<u8>) {
    CHUNK_STORAGE.with(|storage| {
        storage.borrow_mut().insert(chunk_id.to_string(), chunk_data);
    });
}

pub fn get_chunk(chunk_id: &str) -> Option<Vec<u8>> {
    CHUNK_STORAGE.with(|storage| {
        storage.borrow().get(&chunk_id.to_string())
    })
}

pub fn list_models() -> Vec<String> {
    MODEL_STORAGE.with(|storage| {
        storage.borrow().iter().map(|(k, _)| k).collect()
    })
}