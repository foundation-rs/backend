mod identity;
mod authorization;

use std::collections::HashSet;

pub struct SecurityContext {
    subject: String,
    groups:  HashSet<String>,
}

impl SecurityContext {
    pub fn new(subject: String, groups:  HashSet<String>) -> Self {
        Self { subject, groups }
    }
}

pub use identity::IdentityService;
pub use authorization::Authorized;