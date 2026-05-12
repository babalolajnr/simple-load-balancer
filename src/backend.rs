use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

/// Represents a single backend server and its current state.
pub struct Backend {
    pub address: String,
    pub is_healthy: AtomicBool,
    pub active_connections: AtomicUsize,
}

impl Backend {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            is_healthy: AtomicBool::new(true), // Assume healthy initially until proven otherwise
            active_connections: AtomicUsize::new(0),
        }
    }
}

/// A safety guard that automatically decrements the active connection count when dropped.
pub struct ConnectionGuard {
    backends: Arc<Vec<Backend>>,
    backend_index: usize,
}

impl ConnectionGuard {
    pub fn new(backends: Arc<Vec<Backend>>, backend_index: usize) -> Self {
        // Increment the active connection count for the backend
        backends[backend_index]
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
        Self {
            backends,
            backend_index,
        }
    }
}

// When the ConnectionGuard is dropped, decrement the active connection count for the backend.
impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.backends[self.backend_index]
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
    }
}
