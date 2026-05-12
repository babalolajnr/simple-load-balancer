# Load Balancer

A high-performance, async TCP load balancer written in Rust. This project demonstrates core load balancing concepts including multiple routing algorithms, active health checks, and concurrent connection handling.

## Features

- **Multiple Load Balancing Algorithms**
  - Round Robin: Distributes connections sequentially across backends
  - Least Connections: Routes to the backend with the fewest active connections
  - Random: Routes to a randomly selected backend

- **Active Health Checks**: Continuously monitors backend server health with automatic failover
- **Concurrent Connection Handling**: Uses `tokio` for async I/O and efficient resource management
- **Connection Tracking**: Monitors active connections per backend server
- **Bidirectional Data Forwarding**: Transparently proxies data between clients and backend servers

## Architecture

### Core Components

- **`main.rs`**: Entry point that initializes the load balancer and accepts client connections
- **`balancer.rs`**: Core load balancer logic for backend selection
- **`algorithm.rs`**: Implements different load balancing strategies
- **`backend.rs`**: Represents backend servers and tracks their state
- **`health.rs`**: Periodic health check routine that validates backend availability
- **`lib.rs`**: Library exports

### Data Flow

1. Client connects to load balancer on `127.0.0.1:8080`
2. Load balancer selects a healthy backend using the configured algorithm
3. Connection to backend is established
4. Data is bidirectionally forwarded between client and backend
5. When connection closes, connection count is automatically decremented

## Building

Prerequisites:
- Rust 1.70+ (supports edition 2024)
- Cargo

```bash
cargo build --release
```

## Running

### Start the Load Balancer

```bash
cargo run --release -- [OPTIONS]
```

Example with custom settings:
```bash
cargo run --release -- --listen 127.0.0.1:8080 --backends 127.0.0.1:8081,127.0.0.1:8082 --algorithm round-robin
```

Expected output:
```
Load balancer started on 127.0.0.1:8080
Algorithm: "round-robin" | Active health checks: Enabled
```

### Running with Test Servers

Before starting the load balancer, you need backend servers listening on the specified ports.

Example using the provided `test_backends.py` (requires Python 3):
```bash
# Terminal 1: Start backend servers (8081, 8082, 8083)
python3 test_backends.py

# Terminal 2: Start the load balancer
cargo run --release -- --backends 127.0.0.1:8081,127.0.0.1:8082,127.0.0.1:8083
```

### Testing with Python Client

A test client is provided to simulate multiple concurrent connections:

```bash
python3 test_lb.py
```

This script:
- Simulates 10 concurrent clients (configurable)
- Each client connects to the load balancer and sends a message
- Holds connections open for 2 seconds to test connection tracking
- Perfect for testing the "least connections" algorithm

## Configuration

The load balancer is configured via command-line arguments:

| Argument | Short | Default | Description |
|----------|-------|---------|-------------|
| `--listen` | `-l` | `127.0.0.1:8080` | Address to listen on |
| `--backends` | `-b` | `127.0.0.1:8081,127.0.0.1:8082,127.0.0.1:8083` | Comma-separated list of backend addresses |
| `--algorithm` | `-a` | `least-connections` | Load balancing algorithm (`round-robin`, `least-connections`, `random`) |
| `--health-interval` | | `5` | Health check interval in seconds |
| `--health-timeout` | | `1` | Health check timeout in seconds |

Example:
```bash
cargo run -- --backends 127.0.0.1:8081,127.0.0.1:8082 --health-interval 2
```

## Dependencies

- **tokio**: Async runtime with networking support (v1 with "full" features)
- **rand**: Random number generation for the Random algorithm

## How It Works

### Load Balancing Algorithms

#### Round Robin
Each backend is selected in order, cycling through all available backends. Simple and fair distribution.

#### Least Connections
Routes new connections to the backend with the fewest active connections. Ideal for long-lived connections. Prevents overloading busy backends.

#### Random
Randomly selects a backend from all healthy backends. Simple but can lead to uneven distribution.

### Health Checks

The load balancer runs a background task that:
1. Every 5 seconds, attempts a TCP connection to each backend
2. Marks backends as healthy/unhealthy based on connectivity
3. Only routes traffic to healthy backends
4. Logs state changes (healthy → unhealthy or vice versa)

### Connection Tracking

Using a `ConnectionGuard` RAII pattern:
- When a connection is routed to a backend, the connection counter is incremented
- When the connection closes (guard is dropped), the counter is automatically decremented
- The least connections algorithm uses this count to make routing decisions

## Example Output

```
Load balancer started on 127.0.0.1:8080
Algorithm: least-connections | Active health checks: Enabled
Routing 127.0.0.1:52345 -> 127.0.0.1:8081 (Active Conns: 1)
Routing 127.0.0.1:52346 -> 127.0.0.1:8082 (Active Conns: 1)
Routing 127.0.0.1:52347 -> 127.0.0.1:8083 (Active Conns: 1)
Routing 127.0.0.1:52348 -> 127.0.0.1:8081 (Active Conns: 2)
Backend 127.0.0.1:8081 is now healthy! Marking as HEALTHY.
Backend 127.0.0.1:8082 is now unhealthy! Marking as UNHEALTHY.
```

## Performance Considerations

- **Async I/O**: Tokio enables handling thousands of concurrent connections efficiently
- **Connection Pooling**: Consider adding connection pooling to improve performance with frequent reconnections
- **Health Check Optimization**: Currently uses blocking TCP connects with 1-second timeout; consider using non-blocking approaches
- **Scaling**: For production use, consider:
  - Making backends configurable via CLI arguments or config files
  - Adding metrics collection (Prometheus, etc.)
  - Implementing connection reuse
  - Adding rate limiting and circuit breakers

## Future Enhancements

- [ ] Configuration file support (TOML/YAML)
- [x] CLI arguments for dynamic configuration
- [ ] Metrics and monitoring endpoints
- [ ] Connection pooling
- [ ] Weighted round-robin
- [ ] Sticky sessions / session affinity
- [ ] Rate limiting and DDoS protection
- [ ] TLS/HTTPS support
- [ ] Comprehensive logging framework

## License

This project is provided as-is for educational purposes.
