use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub previous_hash: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuditLog {
    events: Vec<AuditEvent>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, action: String) {
        let previous_hash = self.events.last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| "0".repeat(64));

        let timestamp = Utc::now();
        // Use debug format for timestamp to ensure consistency or to_rfc3339
        let payload = format!("{}{}{}", timestamp.to_rfc3339(), action, previous_hash);
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let hash = format!("{:x}", hasher.finalize());

        let event = AuditEvent {
            timestamp,
            action,
            previous_hash,
            hash,
        };
        self.events.push(event);
    }

    pub fn verify(&self) -> bool {
        for (i, event) in self.events.iter().enumerate() {
            let prev_hash = if i == 0 {
                "0".repeat(64)
            } else {
                self.events[i-1].hash.clone()
            };

            if event.previous_hash != prev_hash {
                return false;
            }

            let payload = format!("{}{}{}", event.timestamp.to_rfc3339(), event.action, prev_hash);
            let mut hasher = Sha256::new();
            hasher.update(payload);
            let computed_hash = format!("{:x}", hasher.finalize());

            if event.hash != computed_hash {
                return false;
            }
        }
        true
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_verification() {
        let mut log = AuditLog::new();
        log.append("Action 1".to_string());
        log.append("Action 2".to_string());

        assert!(log.verify());
        assert_eq!(log.len(), 2);

        // Tamper with log
        log.events[0].action = "Tampered Action".to_string();
        assert!(!log.verify());
    }
}
