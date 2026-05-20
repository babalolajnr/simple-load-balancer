use clap::Parser;
use load_balancer::{
    algorithm::Algorithm,
    backend::ConnectionGuard,
    balancer::LoadBalancer,
    config::toml::TomlConfig,
    health::run_health_checks,
    tls::{load_certs, load_keys},
};
use rustls::ServerConfig;
use std::path::Path;
use std::sync::{Arc, atomic::Ordering};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};
use tokio_rustls::TlsAcceptor;

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

    /// Path to TLS certificate (PEM)
    #[arg(long)]
    tls_cert: Option<String>,

    /// Path to TLS private key (PEM)
    #[arg(long)]
    tls_key: Option<String>,
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

    let backends = args.backends.or(config_file.backends).unwrap_or_else(|| {
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

    let health_interval = args
        .health_interval
        .or(config_file.health_interval)
        .unwrap_or(5);
    let health_timeout = args
        .health_timeout
        .or(config_file.health_timeout)
        .unwrap_or(1);

    let tls_cert = args.tls_cert.or(config_file.tls_cert);
    let tls_key = args.tls_key.or(config_file.tls_key);

    let tls_acceptor = if let (Some(cert_path), Some(key_path)) = (tls_cert, tls_key) {
        let certs = load_certs(Path::new(&cert_path))?;
        let mut keys = load_keys(Path::new(&key_path))?;

        if keys.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "No valid private keys found",
            ));
        }

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, keys.remove(0))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Some(TlsAcceptor::from(Arc::new(config)))
    } else {
        None
    };

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
        let (client_stream, client_address) = listener.accept().await?;
        let lb = Arc::clone(&lb);
        let tls_acceptor = tls_acceptor.clone();

        tokio::spawn(async move {
            let mut client_stream: Box<dyn AsyncReadWrite + Unpin + Send> =
                if let Some(acceptor) = tls_acceptor {
                    match acceptor.accept(client_stream).await {
                        Ok(tls_stream) => Box::new(tls_stream),
                        Err(e) => {
                            eprintln!("TLS handshake error from {}: {}", client_address, e);
                            return;
                        }
                    }
                } else {
                    Box::new(client_stream)
                };

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

trait AsyncReadWrite: io::AsyncRead + io::AsyncWrite {}
impl<T: io::AsyncRead + io::AsyncWrite> AsyncReadWrite for T {}
