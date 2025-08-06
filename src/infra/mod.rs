pub mod guards;
pub mod metrics;

use candid::Principal;
use ic_cdk::api::caller;

pub fn get_caller_id() -> String {
    caller().to_text()
}

pub fn is_anonymous() -> bool {
    caller() == Principal::anonymous()
}

pub fn require_authenticated() -> Result<String, String> {
    if is_anonymous() {
        Err("Authentication required".to_string())
    } else {
        Ok(get_caller_id())
    }
}