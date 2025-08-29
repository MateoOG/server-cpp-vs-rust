# Task Processing System

A task processing system with REST API for mathematical calculations.

## Features

- **API Endpoints**: Implements four endpoints
- **Mathematical Operations**: Supports factorial, fibonacci, and prime_check calculations
- **Task Completion Control**: Tasks can ONLY be completed via `POST /task/{id}/complete`
- **Multi-threaded Workers**: Configurable number of workers and threads per worker
- **Round-Robin Load Balancing**: Distributes tasks across workers
- **Comprehensive Testing**: Unit tests, integration tests, and performance tests

## Quick Start

### 1. Install Dependencies

```bash
# Install system dependencies and download required libraries
make install-deps
```

### 2. Build the System

```bash
# Build the application (incremental compilation)
make
```

### 3. Run the System

```bash
# Run with default configuration
make run

# Run with confi.json configuration
make run-config        # Run with config.json

# OR run directly
./build/task_processor

# OR with custom parameters
./build/task_processor --workers 5 --threads 8 --orchestrator-portt 9000
```

### 4. Test the System

```bash
# Run all tests
make test-all

# OR individual test types
make test           # Unit tests
make integration-test  # Integration tests
make perf-test         # Performance tests
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

**Response:**
```json
{
  "total_workers": 3,
  "total_tasks_processed": 150,
  "total_tasks_completed": 145,
  "total_tasks_failed": 5,
  "uptime_seconds": 3600
}
```
## Supported Operations

The system supports exactly three mathematical operations:

### factorial
Calculate factorial of a number (0-20).
```json
{"operation": "factorial", "input": 5}  // Returns "120"
```

### fibonacci  
Calculate nth Fibonacci number (0-1000).
```json
{"operation": "fibonacci", "input": 10}  // Returns "55"
```

### prime_check
Check if a number is prime (≥2).
```json
{"operation": "prime_check", "input": 17}  // Returns "true"
```

## Task Processing Workflow

1. **Create Task** → Status: `pending`
2. **Worker Processes** → Status: `processing` (result calculated and stored)
3. **API Completion Call** → Status: `completed` (via `POST /task/{id}/complete`)

**Important:** Tasks remain in `processing` status until explicitly completed via the API endpoint.

## Configuration

### Command Line Options

```bash
./build/task_processor [OPTIONS]

Options:
  -w, --workers NUM          Number of workers (default: 3, max: 50)
  -t, --threads NUM          Threads per worker (default: 4, max: 32)  
  -p, --port NUM             Base port for workers (default: 8080)
  -o, --orchestrator-port NUM Orchestrator port (default: 7000)
  -c, --config FILE          Configuration file (JSON)
  -h, --help                 Show help message
```

### Configuration File (config.json)

```json
{
  "num_workers": 3,
  "threads_per_worker": 4,
  "orchestrator_port": 7000,
}
```

## Build System

### Makefile Targets

```bash
# Dependencies (run once)
make install-deps      # Install system dependencies
make install-httplib   # Download cpp-httplib header

# Build (incremental compilation)
make                   # Default build
make debug             # Debug build with symbols
make release           # Optimized release build

# Testing
make test              # Unit tests
make integration-test  # Integration tests  
make perf-test         # Performance tests
make test-all          # All tests

# Run
make run               # Run with defaults
make run-config        # Run with config.json
make run-custom        # Run with custom parameters

# Documentation
make docs              # Generate Doxygen documentation

# Cleanup
make clean             # Remove build artifacts (keep dependencies)
make clean-all         # Remove everything including dependencies
```
## Testing

### Unit Tests (C++ with Google Test)
```bash
make test
# OR
./build/test_runner
```

### Integration Tests (Python)
```bash
make integration-test
# OR  
python3 tests/integration_test.py
```

### Performance Tests (Python)
```bash
make perf-test
# OR
python3 tests/performance_test.py --priority-tasks 50
```

## Architecture

```
┌─────────────────┐    ┌─────────────────┐
│   Orchestrator  │    │     Client      │
│   Port: 7000    │◄──►│   Applications  │
└─────────────────┘    └─────────────────┘
         │
         │ Round-Robin
         ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│    Worker 0     │    │    Worker 1     │    │    Worker N     │
│   Port: 8080    │    │   Port: 8081    │    │  Port: 8080+N   │
│                 │    │                 │    │                 │  
│   Thread 1      │    │   Thread 1      │    │   Thread 1      │
│   Thread 2      │    │   Thread 2      │    │   Thread 2      │
│   Thread N      │    │   Thread N      │    │   Thread N      │
└─────────────────┘    └─────────────────┘    └─────────────────┘

```

## Dependencies

- **C++17** compiler (g++ or clang++)
- **cmake** (≥3.12) or **make**
- **nlohmann/json** (JSON library)
- **cpp-httplib** (HTTP library - auto-downloaded)
- **Google Test** (for unit tests)
- **Python 3** + **aiohttp**, **requests** (for integration/performance tests)
- **Doxygen** (for documentation generation)

## Examples

### Create and Complete a Task

```bash
# 1. Create a high-priority factorial task
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

# 2. Check task status (should be "processing" with result)
curl http://localhost:7000/task/example-001

# 3. Complete the task (REQUIRED step)  
curl -X POST http://localhost:7000/task/example-001/complete

# 4. Verify completion
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
├── config.json
├── Doxyfile
├── Makefile
├── manual_test_rs.sh
├── Readme.md
├── src
│   ├── calculations.cpp
│   ├── calculations.h
│   ├── httplib.h
│   ├── main.cpp
│   ├── orchestrator.cpp
│   ├── orchestrator.h
│   ├── task.cpp
│   ├── task.h
│   ├── worker.cpp
│   └── worker.h
└── tests
    ├── integration_test.py
    ├── performance_test.py
    └── unit_tests.cpp

2 directories, 18 files
```

### Coding Standards

- **C++17** standard
- **Doxygen** documentation for all public APIs
- **Google Test** for unit testing
- **RAII** and modern C++ practices
- **Thread-safe** design patterns

## Troubleshooting

### Build Issues

1. **Missing dependencies**: Run `make install-deps`
2. **httplib.h not found**: Run `make install-httplib` 
3. **Compilation errors**: Check C++17 compiler support

### Runtime Issues

1. **Port conflicts**: Change ports in config.json or command line
2. **Connection refused**: Ensure system is started and ports are available
3. **Tasks not processing**: Check worker thread configuration
4. **Tasks stuck in processing**: Use `POST /task/{id}/complete` to finish

### Performance Issues

1. **Low throughput**: Increase worker/thread count
2. **High latency**: Check system resources and task complexity  
3. **Memory usage**: Monitor task queue sizes and completion rates

## License

Define later.
