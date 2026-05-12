use load_balancer::{
    algorithm::Algorithm, backend::ConnectionGuard, balancer::LoadBalancer,
    health::run_health_checks,
};
use std::sync::{Arc, atomic::Ordering};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listen_addr = "127.0.0.1:8080";
    let algorithm = Algorithm::LeastConnections;

    let lb = Arc::new(LoadBalancer::new(
        vec!["127.0.0:8081", "127.0.0:8082", "127.0.0:8083"],
        algorithm,
    ));

    let lb_clone = Arc::clone(&lb);
    tokio::spawn(async move {
        run_health_checks(Arc::clone(&lb_clone.backends)).await;
    });

    let listener = TcpListener::bind(listen_addr).await?;
    println!("Load balancer started on {}", listen_addr);
    println!(
        "Algorithm: {:?} | Active health checks: Enabled",
        algorithm.as_str()
    );

    loop {
        let (mut client_stream, client_address) = listener.accept().await?;
        let lb = Arc::clone(&lb);

        tokio::spawn(async move {
            if let Some(backend_index) = lb.select_backend() {
                let backend_address = lb.backends[backend_index].address.clone();

                let _guard = ConnectionGuard::new(Arc::clone(&lb.backends), backend_index);

                match TcpStream::connect(&backend_address).await {
                    Ok(mut server_stream) => {
                        println!(
                            "Routing {} -> {} (Active Conns: {})",
                            client_address,
                            backend_address,
                            lb.backends[backend_index]
                                .active_connections
                                .load(Ordering::Relaxed)
                        );

                        if let Err(e) =
                            io::copy_bidirectional(&mut client_stream, &mut server_stream).await
                        {
                            eprintln!("Data stream error: {}", e);
                        }
                    }
                    Err(_) => {
                        eprintln!("Failed to connect to chosen backend: {}", backend_address);
                    }
                }
            } else {
                eprintln!(
                    "503 SERVICE UNAVAILABLE: No healthy backends to route {} to!",
                    client_address
                );
            }
        });
    }
}
