use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use tokio::{net::TcpStream, time};

use crate::backend::Backend;

/// Runs infinitely in the background, pinging servers to verify they are online
pub async fn run_health_checks(
    backends: Arc<Vec<Backend>>,
    interval_secs: u64,
    timeout_secs: u64,
) {
    let mut interval = time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        for backend in backends.iter() {
            // use a timeout to prevent health checker from hanging
            match tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                TcpStream::connect(&backend.address),
            )
            .await
            {
                Ok(Ok(_)) => {
                    let was_healthy = backend.is_healthy.swap(true, Ordering::Relaxed);
                    if !was_healthy {
                        println!(
                            "Backend {} is now healthy! Marking as HEALTHY.",
                            backend.address
                        );
                    }
                }
                _ => {
                    let was_healthy = backend.is_healthy.swap(false, Ordering::Relaxed);
                    if was_healthy {
                        println!(
                            "Backend {} is now unhealthy! Marking as UNHEALTHY.",
                            backend.address
                        );
                    }
                }
            }
        }
    }
}
