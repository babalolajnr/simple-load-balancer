use clap::Parser;
use load_balancer::{
    algorithm::Algorithm, backend::ConnectionGuard, balancer::LoadBalancer,
    health::run_health_checks, config::toml::TomlConfig,
};
use std::sync::{Arc, atomic::Ordering};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML)
    #[arg(short, long)]
    config: Option<String>,

    /// Address to listen on
    #[arg(short, long)]
    listen: Option<String>,

    /// Backend servers to route traffic to
    #[arg(short, long, value_delimiter = ',')]
    backends: Option<Vec<String>>,

    /// Load balancing algorithm to use
    #[arg(short, long)]
    algorithm: Option<Algorithm>,

    /// Health check interval in seconds
    #[arg(long)]
    health_interval: Option<u64>,

    /// Health check timeout in seconds
    #[arg(long)]
    health_timeout: Option<u64>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let config_file = if let Some(path) = &args.config {
        TomlConfig::new(path).unwrap_or_else(|e| {
            eprintln!("Failed to load config file: {}", e);
            std::process::exit(1);
        })
    } else {
        TomlConfig::default()
    };

    let listen = args
        .listen
        .or(config_file.listen)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let backends = args
        .backends
        .or(config_file.backends)
        .unwrap_or_else(|| {
            vec![
                "127.0.0.1:8081".to_string(),
                "127.0.0.1:8082".to_string(),
                "127.0.0.1:8083".to_string(),
            ]
        });

    let algorithm = args
        .algorithm
        .or(config_file.algorithm)
        .unwrap_or(Algorithm::LeastConnections);

    let health_interval = args.health_interval.or(config_file.health_interval).unwrap_or(5);
    let health_timeout = args.health_timeout.or(config_file.health_timeout).unwrap_or(1);

    let lb = Arc::new(LoadBalancer::new(backends.clone(), algorithm));

    let lb_clone = Arc::clone(&lb);
    tokio::spawn(async move {
        run_health_checks(
            Arc::clone(&lb_clone.backends),
            health_interval,
            health_timeout,
        )
        .await;
    });

    let listener = TcpListener::bind(&listen).await?;
    println!("Load balancer started on {}", listen);
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
