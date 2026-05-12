use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::{algorithm::Algorithm, backend::Backend};

/// Manages pool of backends and routes traffic to them using a specified algorithm.
pub struct LoadBalancer {
    pub backends: Arc<Vec<Backend>>,
    algorithm: Algorithm,
    current_index: AtomicUsize,
}

impl LoadBalancer {
    pub fn new(backends_addresses: Vec<String>, algorithm: Algorithm) -> Self {
        let backends = backends_addresses
            .into_iter()
            .map(Backend::new)
            .collect::<Vec<_>>();

        Self {
            backends: Arc::new(backends),
            algorithm,
            current_index: AtomicUsize::new(0),
        }
    }

    pub fn select_backend(&self) -> Option<usize> {
        let healthy_indices = self
            .backends
            .iter()
            .enumerate()
            .filter(|(_, b)| b.is_healthy.load(Ordering::Relaxed))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        if healthy_indices.is_empty() {
            return None;
        }

        match self.algorithm {
            Algorithm::RoundRobin => {
                let count = self.current_index.fetch_add(1, Ordering::Relaxed);
                let chosen_index = healthy_indices[count % healthy_indices.len()];
                Some(chosen_index)
            }
            Algorithm::Random => {
                let rng = rand::random::<f64>() as usize % healthy_indices.len();
                Some(healthy_indices[rng])
            }
            Algorithm::LeastConnections => {
                let chosen_index = healthy_indices
                    .into_iter()
                    .min_by_key(|&i| self.backends[i].active_connections.load(Ordering::Relaxed))
                    .unwrap();
                Some(chosen_index)
            }
        }
    }
}
