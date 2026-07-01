//! Canary Token System — MICAFP v10.0 Feature 19

pub const CANARY_UID_PREFIX: &str = "canary_";

pub fn is_canary_uid(uid: &str) -> bool {
    uid.starts_with(CANARY_UID_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_canary_detection() {
        assert!(is_canary_uid("canary_001"));
        assert!(!is_canary_uid("user_abc123"));
    }
}
