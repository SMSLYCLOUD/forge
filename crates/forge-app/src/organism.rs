use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Organism state that the UI reads from
/// This is updated by background threads; UI only reads
#[derive(Clone, Debug)]
pub struct OrganismState {
    /// Overall confidence score (0.0 - 100.0)
    pub confidence_score: f32,
    /// Per-line confidence (line_index → score)
    #[allow(dead_code)]
    pub line_confidence: Vec<f32>,
    /// Anticipation predictions
    #[allow(dead_code)]
    pub predictions: Vec<Prediction>,
    /// Last heartbeat time
    pub last_heartbeat: Instant,
    /// Whether the organism is running
    pub alive: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Prediction {
    pub action: String,
    pub probability: f32,
    pub pre_warm: bool,
}

impl OrganismState {
    pub fn new() -> Self {
        Self {
            confidence_score: 0.0,
            line_confidence: Vec::new(),
            predictions: Vec::new(),
            last_heartbeat: Instant::now(),
            alive: false,
        }
    }
}

impl Default for OrganismState {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe handle to organism state
pub type SharedOrganismState = Arc<Mutex<OrganismState>>;

/// Create a new shared organism state
pub fn new_shared_state() -> SharedOrganismState {
    Arc::new(Mutex::new(OrganismState::new()))
}

/// Start the organism heartbeat on a background thread
/// This thread periodically updates the organism state
pub fn start_heartbeat(
    state: SharedOrganismState,
    heartbeat_interval: Duration,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(heartbeat_interval);

            if let Ok(mut s) = state.lock() {
                s.last_heartbeat = Instant::now();
                s.alive = true;

                // TODO: Connect to actual forge-confidence crate
                // For now, set a default confidence score
                if s.confidence_score < 0.01 {
                    s.confidence_score = 75.0;
                }
            }
        }
    })
}

/// Read organism state safely (never blocks UI for more than a few microseconds)
#[allow(dead_code)]
pub fn read_state(state: &SharedOrganismState) -> Option<OrganismState> {
    // try_lock to avoid blocking the render thread
    state.try_lock().ok().map(|guard| guard.clone())
}
