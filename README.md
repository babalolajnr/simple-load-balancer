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
cargo run --release
```

Expected output:
```
Load balancer started on 127.0.0.1:8080
Algorithm: least-connections | Active health checks: Enabled
```

### Running with Test Servers

Before starting the load balancer, you need backend servers listening on ports 8081, 8082, and 8083. You can use simple TCP echo servers or any service listening on those ports.

Example with `nc` (netcat):
```bash
# Terminal 1: Start backend 1
nc -l 127.0.0.1 8081

# Terminal 2: Start backend 2
nc -l 127.0.0.1 8082

# Terminal 3: Start backend 3
nc -l 127.0.0.1 8083

# Terminal 4: Start the load balancer
cargo run --release
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

#### Test Script Configuration

In `test_lb.py`, you can adjust:
- `NUM_CLIENTS`: Number of concurrent clients (default: 10)
- `CONNECTION_DELAY`: Delay between starting clients (default: 0.5s)
- `WORK_DURATION`: How long each client stays connected (default: 2s)

## Configuration

Currently, configuration is hardcoded in `src/main.rs`. To customize:

### Change Listen Address
```rust
let listen_addr = "127.0.0.1:8080";  // Modify this line
```

### Change Algorithm
```rust
let algorithm = Algorithm::LeastConnections;  // Change to RoundRobin or Random
```

### Change Backend Servers
```rust
vec!["127.0.0:8081", "127.0.0:8082", "127.0.0:8083"]  // Modify addresses
```

### Adjust Health Check Interval
In `src/health.rs`, modify:
```rust
let mut interval = time::interval(Duration::from_secs(5));  // Change interval
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
- [ ] CLI arguments for dynamic configuration
- [ ] Metrics and monitoring endpoints
- [ ] Connection pooling
- [ ] Weighted round-robin
- [ ] Sticky sessions / session affinity
- [ ] Rate limiting and DDoS protection
- [ ] TLS/HTTPS support
- [ ] Comprehensive logging framework

## License

This project is provided as-is for educational purposes.
