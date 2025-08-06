use ic_cdk::api::caller;
use std::collections::HashSet;

pub struct RateLimiter {
    requests_per_minute: HashMap<String, u32>,
    limits: HashMap<String, u32>, // principal -> limit
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            requests_per_minute: HashMap::new(),
            limits: HashMap::new(),
        }
    }

    pub fn check_rate_limit(&mut self, principal: &str) -> Result<(), String> {
        let limit = self.limits.get(principal).unwrap_or(&60); // Default 60/min
        let current = self.requests_per_minute.get(principal).unwrap_or(&0);
        
        if current >= limit {
            return Err("Rate limit exceeded".to_string());
        }
        
        self.requests_per_minute.insert(principal.to_string(), current + 1);
        Ok(())
    }
    
    pub fn set_limit(&mut self, principal: String, limit: u32) {
        self.limits.insert(principal, limit);
    }
}

use std::collections::HashMap;

thread_local! {
    static RATE_LIMITER: std::cell::RefCell<RateLimiter> = std::cell::RefCell::new(RateLimiter::new());
}

pub fn check_rate_limit() -> Result<(), String> {
    let principal = caller().to_text();
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_rate_limit(&principal)
    })
}

pub fn is_authorized_caller(authorized_principals: &[String]) -> Result<String, String> {
    let caller_id = caller().to_text();
    
    if authorized_principals.contains(&caller_id) {
        Ok(caller_id)
    } else {
        Err("Caller not authorized".to_string())
    }
}