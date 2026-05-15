use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use serde::Deserialize;

/// Represents a single backend server and its current state.
#[derive(Debug, Deserialize)]
pub struct Backend {
    pub address: String,
    pub is_healthy: AtomicBool,
    pub active_connections: AtomicUsize,
}

impl Backend {
    pub fn new(address: String) -> Self {
        Self {
            address,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_connection_guard_creation_and_drop() {
        let backends = Arc::new(vec![Backend::new("127.0.0.1:8080".to_string())]);

        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 0);

        {
            let _guard = ConnectionGuard::new(backends.clone(), 0);
            assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 1);
        }

        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_multiple_connection_guards_same_backend() {
        let backends = Arc::new(vec![Backend::new("127.0.0.1:8080".to_string())]);

        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 0);

        let guard1 = ConnectionGuard::new(backends.clone(), 0);
        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 1);

        let guard2 = ConnectionGuard::new(backends.clone(), 0);
        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 2);

        drop(guard1);
        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 1);

        drop(guard2);
        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_connection_guards_different_backends() {
        let backends = Arc::new(vec![
            Backend::new("127.0.0.1:8080".to_string()),
            Backend::new("127.0.0.1:8081".to_string()),
        ]);

        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 0);
        assert_eq!(backends[1].active_connections.load(Ordering::Relaxed), 0);

        let guard1 = ConnectionGuard::new(backends.clone(), 0);
        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 1);
        assert_eq!(backends[1].active_connections.load(Ordering::Relaxed), 0);

        let guard2 = ConnectionGuard::new(backends.clone(), 1);
        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 1);
        assert_eq!(backends[1].active_connections.load(Ordering::Relaxed), 1);

        drop(guard1);
        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 0);
        assert_eq!(backends[1].active_connections.load(Ordering::Relaxed), 1);

        drop(guard2);
        assert_eq!(backends[0].active_connections.load(Ordering::Relaxed), 0);
        assert_eq!(backends[1].active_connections.load(Ordering::Relaxed), 0);
    }
}
