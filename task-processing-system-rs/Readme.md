# Task Processing System (Rust)

Task processing system with REST API for mathematical calculations.

## Features

- **Mathematical Operations**: Supports factorial, fibonacci, and prime_check calculations
- **Task Completion Control**: Tasks can ONLY be completed via `POST /task/{id}/complete`
- **Multi-threaded Workers**: Configurable number of workers and threads per worker
- **Round-Robin Load Balancing**: Distributes tasks across workers
- **Memory Safety**: Built with Rust's ownership system for zero-cost abstractions and memory safety
- **Async/Await**: Full async support with Tokio runtime for high concurrency

## Quick Start

### 1. Install Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Build the System

```bash
# Development build (fast compilation, debug symbols)
cargo build

# Release build (optimized for performance)
cargo build --release

# Build with all features
cargo build --release --all-features
```

### 3. Run the System

```bash
# Run with default configuration
cargo run

# Run with configuration file
cargo run -- --config config.toml

# Run with custom parameters
cargo run -- --workers 5 --threads 8 --orchestrator-port 9000

# Run release build
cargo run --release
```

### 4. Test the System

```bash
# Run all (unit and integration) tests
cargo test

# Run integration tests
cargo test --test integration_test

# Run all tests with output
cargo test -- --nocapture

# Run performance benchmarks
cargo bench
```

## API Endpoints

The system implements four endpoints:

### Orchestrator API (`http://localhost:7000`)

#### POST /task/create
Create a new task.

```json
{
  "id": "task-001",
  "title": "Process calculation", 
  "priority": 3,
  "data": {
    "type": "calculation",
    "input": 10,
    "operation": "factorial"
  }
}
```

**Priority Levels:**
- `1` = LOW priority
- `2` = MEDIUM priority (default)
- `3` = HIGH priority

#### GET /task/{id}
Get task information.

**Response:**
```json
{
  "id": "task-001",
  "title": "Process calculation",
  "priority": 3,
  "status": "processing",
  "result": "3628800",
  "created_at": "2024-01-15T10:30:00Z"
}
```

**Status Values:**
- `pending` - Task created, waiting to be processed
- `processing` - Task calculation completed, awaiting API completion
- `completed` - Task marked complete via API call
- `failed` - Task processing failed

#### POST /task/{id}/complete
Mark task as completed (**ONLY** way to complete tasks).

**Response:**
```json
{
  "id": "task-001",
  "status": "completed",
  "message": "Task completed successfully"
}
```

#### GET /stats
Get worker statistics.

```json
{
  "total_tasks_processed": 42,
  "total_tasks_completed": 38,
  "total_tasks_failed": 1,
  "total_workers": 3,
  "uptime_seconds": 3600,
  "workers": [
    {
      "id": 0,
      "port": 8080,
      "tasks_processed": 15,
      "tasks_completed": 13,
      "tasks_failed": 0,
      "current_load": 2,
      "uptime_seconds": 3600,
      "is_healthy": true
    }
  ]
}
```

## Usage Examples

### Basic Task Creation

```bash
# Create a factorial task
curl -X POST http://localhost:7000/task/create \
  -H "Content-Type: application/json" \
  -d '{
    "id": "example-001", 
    "title": "Calculate 10!",
    "priority": 3,
    "data": {
      "type": "calculation",
      "input": 10, 
      "operation": "factorial"
    }
  }'

# Check task status (should be "processing" with result)
curl http://localhost:7000/task/example-001

# Complete the task (REQUIRED step)  
curl -X POST http://localhost:7000/task/example-001/complete

# Verify completion
curl http://localhost:7000/task/example-001
```

### Check System Statistics

```bash
curl http://localhost:7000/stats
```

## Development

### Project Structure

```
.
├── benches
│   └── task_benchmarks.rs
├── Cargo.lock
├── Cargo.toml
├── config.toml
├── examples
│   ├── client_example.rs
│   └── load_test.rs
├── manual_test_rs.sh
├── Readme.md
├── src
│   ├── calculations.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── orchestrator.rs
│   ├── types.rs
│   └── worker.rs
└── tests
    └── integration_tests.rs

4 directories, 17 files
```

### Cargo Commands

```bash
# Development workflow
cargo check                # Fast compilation check
cargo build               # Debug build
cargo build --release     # Optimized build
cargo run                 # Run with default config
cargo test                # Run tests
cargo bench               # Run benchmarks
cargo doc --open          # Generate and open documentation

# Code quality
cargo clippy              # Linting
cargo fmt                 # Code formatting
cargo audit               # Security audit

# Release management
cargo publish --dry-run   # Test publishing
cargo publish             # Publish to crates.io
```

### Configuration

The system can be configured via:

1. **Command line arguments** (highest priority)
2. **Configuration files** (`config.toml`)
3. **Default values** (lowest priority)

Example `config.toml`:
```toml
num_workers = 3
threads_per_worker = 4
base_port = 8080
orchestrator_port = 7000
log_level = "info"
```

### Environment Variables

```bash
#Run the server with info prints:
RUST_LOG=info cargo run

# Rust logging
RUST_LOG=debug cargo run

# Custom log levels per module
RUST_LOG=task_processing_system=debug,warp=info cargo run
```
## Troubleshooting

### Build Issues

1. **Rust not installed**: Install via rustup as shown above
2. **Compilation errors**: Run `cargo check` for faster error checking
3. **Dependency issues**: Run `cargo update` to update dependencies

### Runtime Issues

1. **Port already in use**: Change ports in config or use `--orchestrator-port`
2. **Permission denied**: Use ports > 1024
3. **High CPU usage**: Reduce `num_workers` or `threads_per_worker`

### Performance Issues

1. **Low throughput**: Increase `num_workers` and `threads_per_worker`
2. **High latency**: Check system load and reduce concurrent tasks
3. **Memory issues**: Monitor with `htop` and adjust configuration

## License

Define later.

## Support

- **Documentation**: Run `cargo doc --open` for full API documentation
- **Performance**: Use `cargo bench` for performance testing
