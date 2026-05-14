use clap::Parser;
use load_balancer::{
    algorithm::Algorithm, backend::ConnectionGuard, balancer::LoadBalancer,
    health::run_health_checks,
};
use std::sync::{Arc, atomic::Ordering};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to listen on
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    listen: String,

    /// Backend servers to route traffic to
    #[arg(short, long, default_value = "127.0.0.1:8081,127.0.0.1:8082,127.0.0.1:8083", value_delimiter = ',')]
    backends: Vec<String>,

    /// Load balancing algorithm to use
    #[arg(short, long, default_value = "least-connections")]
    algorithm: Algorithm,

    /// Health check interval in seconds
    #[arg(long, default_value_t = 5)]
    health_interval: u64,

    /// Health check timeout in seconds
    #[arg(long, default_value_t = 1)]
    health_timeout: u64,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let lb = Arc::new(LoadBalancer::new(args.backends.clone(), args.algorithm));

    let lb_clone = Arc::clone(&lb);
    let health_interval = args.health_interval;
    let health_timeout = args.health_timeout;
    tokio::spawn(async move {
        run_health_checks(
            Arc::clone(&lb_clone.backends),
            health_interval,
            health_timeout,
        )
        .await;
    });

    let listener = TcpListener::bind(&args.listen).await?;
    println!("Load balancer started on {}", args.listen);
    println!(
        "Algorithm: {:?} | Active health checks: Enabled",
        args.algorithm.as_str()
    );

    loop {
        let (mut client_stream, client_address) = listener.accept().await?;
        let lb = Arc::clone(&lb);

        tokio::spawn(async move {
            if let Some(backend_index) = lb.select_backend() {
                let backend_address = &lb.backends[backend_index].address;

                let _guard = ConnectionGuard::new(Arc::clone(&lb.backends), backend_index);

                match TcpStream::connect(backend_address).await {
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
