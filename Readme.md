# Task Processing System: C++ vs Rust Comparison

A comparative implementation of a multi-threaded task processing system with REST API, built in both **C++** and **Rust**.

## Quick Start

### 1. Clone the repository:
```
git clone git@github.com:MateoOG/server-cpp-vs-rust.git
cd server-cpp-vs-rust.git
```

### 2. Run the Full Comparison

```bash
# Make the script executable
chmod +x build_cpp_vs_rust.sh

# Run the complete comparison
./build_cpp_vs_rust.sh
```

## What the Script Does

The `build_cpp_vs_rust.sh` script performs the following operations:

1. **Builds both systems** (C++ and Rust)
2. **Generates documentation** for both implementations
3. **Starts servers** in separate terminal windows
4. **Executes performance comparison** between C++ and Rust implementations


## Project Structure

```
project/
├── build_cpp_vs_rust.sh           # Main comparison script
├── perf_test_cpp_vs_rust.py        # Performance comparison tool
├── task-processing-system-cpp/     # C++ implementation
└── task-processing-system-rs/      # Rust implementation
```

## Individual System Commands

### C++ System (`task-processing-system-cpp/`)

```bash
cd task-processing-system-cpp/

# Install dependencies and build
make install-deps
make

# Run the system
make run                    # Default configuration
make run-config            # With config.json

# Run tests
make test-all              # All tests
make test                  # Unit tests only
make integration-test      # Integration tests only

# Generate documentation
make docs
```

**API Endpoint:** `http://localhost:5000`  
**Detailed documentation:** `task-processing-system-cpp/Readme.md`

### Rust System (`task-processing-system-rs/`)

```bash
cd task-processing-system-rs/

# Build
cargo build                # Debug build
cargo build --release     # Optimized build

# Run the system
cargo run                  # Default configuration
cargo run -- --config config.toml

# Run tests
cargo test                 # All tests
cargo bench               # Performance benchmarks

# Generate documentation
cargo doc --open
```

**API Endpoint:** `http://localhost:7000`  
**Detailed documentation:** `task-processing-system-rs/Readme.md`

## API Usage Example (modify port as needed)

```bash
# Create a task
curl -X POST http://localhost:7000/task/create \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-001",
    "title": "Calculate 10!",
    "priority": 3,
    "data": {
      "type": "calculation",
      "input": 10,
      "operation": "factorial"
    }
  }'

# Check task status
curl http://localhost:7000/task/test-001

# Complete the task (required)
curl -X POST http://localhost:7000/task/test-001/complete

# View system statistics
curl http://localhost:7000/stats
```
