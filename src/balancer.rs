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
            Algorithm::RoundRobin => self.round_robin(&healthy_indices),
            Algorithm::Random => self.random(&healthy_indices),
            Algorithm::LeastConnections => self.least_connections(&healthy_indices),
        }
    }

    /// Selects a backend using the round-robin algorithm.
    fn round_robin(&self, healthy_indices: &[usize]) -> Option<usize> {
        let count = self.current_index.fetch_add(1, Ordering::Relaxed);
        let chosen_index = healthy_indices[count % healthy_indices.len()];
        Some(chosen_index)
    }

    /// Selects a backend using the random algorithm.
    fn random(&self, healthy_indices: &[usize]) -> Option<usize> {
        let rng = rand::random_range(0..healthy_indices.len());
        Some(healthy_indices[rng])
    }

    /// Selects a backend using the least-connections algorithm.
    fn least_connections(&self, healthy_indices: &[usize]) -> Option<usize> {
        let chosen_index = healthy_indices
            .iter()
            .min_by_key(|&i| self.backends[*i].active_connections.load(Ordering::Relaxed))
            .unwrap();

        Some(*chosen_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_new_load_balancer_initialization() {
        let addresses = vec!["127.0.0.1:8081".to_string(), "127.0.0.1:8082".to_string()];
        let lb = LoadBalancer::new(addresses.clone(), Algorithm::RoundRobin);

        assert_eq!(lb.backends.len(), 2);
        assert_eq!(lb.backends[0].address, addresses[0]);
        assert_eq!(lb.backends[1].address, addresses[1]);

        for backend in lb.backends.iter() {
            assert!(backend.is_healthy.load(Ordering::Relaxed));
            assert_eq!(backend.active_connections.load(Ordering::Relaxed), 0);
        }

        assert_eq!(lb.current_index.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_new_load_balancer_empty_backends() {
        let addresses: Vec<String> = vec![];
        let lb = LoadBalancer::new(addresses, Algorithm::RoundRobin);

        assert_eq!(lb.backends.len(), 0);
        assert!(lb.select_backend().is_none());
    }

    #[test]
    fn test_new_load_balancer_algorithm_round_robin_start() {
        let addresses = vec!["127.0.0.1:8081".to_string(), "127.0.0.1:8082".to_string()];
        let lb = LoadBalancer::new(addresses, Algorithm::RoundRobin);

        // Initially current_index is 0, so select_backend should return 0 (if healthy)
        assert_eq!(lb.select_backend(), Some(0));
        assert_eq!(lb.current_index.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_random_algorithm_is_actually_random() {
        let backends = vec![
            "127.0.0.1:8081".to_string(),
            "127.0.0.1:8082".to_string(),
            "127.0.0.1:8083".to_string(),
        ];
        let balancer = LoadBalancer::new(backends, Algorithm::Random);

        let mut selected_indices = HashSet::new();
        // With 3 backends, after 100 iterations, we should have seen more than one.
        // Statistically, the chance of picking the same one 100 times is (1/3)^99 which is negligible.
        for _ in 0..100 {
            if let Some(index) = balancer.select_backend() {
                selected_indices.insert(index);
            }
        }

        assert!(
            selected_indices.len() > 1,
            "Random algorithm should select more than one backend, but got only {:?}",
            selected_indices
        );
    }
}
