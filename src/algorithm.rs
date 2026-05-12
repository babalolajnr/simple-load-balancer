/// Defines the available load balancing strategies.
#[derive(Clone, Copy, Debug)]
pub enum Algorithm {
    RoundRobin,
    LeastConnections,
    Random,
}

impl Algorithm {
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::RoundRobin => "round-robin",
            Algorithm::LeastConnections => "least-connections",
            Algorithm::Random => "random",
        }
    }
}
