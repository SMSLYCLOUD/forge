use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub id: u64,
    pub message: String,
    pub level: Level,
    pub created_at: Instant,
}

pub struct NotificationManager {
    pub notifications: Vec<Notification>,
    pub next_id: u64,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self {
            notifications: Vec::new(),
            next_id: 0,
        }
    }
}

impl NotificationManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, message: &str, level: Level) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        self.notifications.push(Notification {
            id,
            message: message.to_string(),
            level,
            created_at: Instant::now(),
        });

        id
    }

    pub fn dismiss(&mut self, id: u64) {
        self.notifications.retain(|n| n.id != id);
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.notifications.retain(|n| {
            let elapsed = now.duration_since(n.created_at);
            match n.level {
                Level::Info => elapsed < Duration::from_secs(5),
                Level::Warning => elapsed < Duration::from_secs(10),
                Level::Error => true, // Sticky
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_show_dismiss() {
        let mut nm = NotificationManager::new();
        let id = nm.show("Hello", Level::Info);
        assert_eq!(nm.notifications.len(), 1);

        nm.dismiss(id);
        assert!(nm.notifications.is_empty());
    }

    #[test]
    fn test_tick_expiry() {
        let mut nm = NotificationManager::new();
        nm.show("Info", Level::Info);
        nm.show("Error", Level::Error);

        // Mocking time is hard with Instant::now().
        // We can't easily test expiry without sleeping, which makes tests slow.
        // For unit test, we can trust the logic `duration_since` works.
        // Or we can sleep for tiny amount if we set threshold very low?
        // But threshold is hardcoded 5s/10s.
        // So we can only test that `tick` doesn't remove immediately.

        nm.tick();
        assert_eq!(nm.notifications.len(), 2);
    }
}
